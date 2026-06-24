/**
 * File Mention Dropdown Component
 *
 * Shows a dropdown list of files when user types @ in the input box.
 * Supports arrow key navigation and Enter key selection.
 */

import { useEffect, useRef } from 'react';
import { FileText } from 'lucide-react';

interface FileItem {
  path: string;
  name: string;
  is_file: boolean;
}

interface FileMentionDropdownProps {
  files: FileItem[];
  selectedIndex: number;
  onSelect: (file: FileItem) => void;
  onClose: () => void;
  position: { top: number; left: number };
}

export function FileMentionDropdown({
  files,
  selectedIndex,
  onSelect,
  onClose,
  position
}: FileMentionDropdownProps) {
  const dropdownRef = useRef<HTMLDivElement>(null);
  const selectedItemRef = useRef<HTMLDivElement>(null);

  // Scroll selected item into view
  useEffect(() => {
    if (selectedItemRef.current) {
      selectedItemRef.current.scrollIntoView({
        block: 'nearest',
        behavior: 'smooth'
      });
    }
  }, [selectedIndex]);

  // Handle click outside to close
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [onClose]);

  if (files.length === 0) {
    return (
      <div
        ref={dropdownRef}
        className="fixed z-50 bg-surface-primary border border-border-emphasis rounded-lg shadow-lg"
        style={{ top: position.top, left: position.left }}
      >
        <div className="px-4 py-3 text-sm text-content-tertiary">
          No files found
        </div>
      </div>
    );
  }

  return (
    <div
      ref={dropdownRef}
      className="fixed z-50 bg-surface-primary border border-border-emphasis rounded-lg shadow-lg w-96 max-h-64 overflow-y-auto"
      style={{ top: position.top, left: position.left }}
    >
      {files.map((file, index) => (
        <div
          key={file.path}
          ref={index === selectedIndex ? selectedItemRef : null}
          onClick={() => onSelect(file)}
          className={`flex items-center gap-3 px-4 py-2.5 cursor-pointer transition-colors ${
            index === selectedIndex
              ? 'bg-surface-2'
              : 'hover:bg-surface-elevated'
          }`}
        >
          <FileText className="w-4 h-4 text-content-tertiary flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <div className="text-sm font-medium text-content-primary truncate">
              {file.name}
            </div>
            <div className="text-xs text-content-tertiary truncate">
              {file.path}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

/**
 * ESLint rule: eventBridge.on first argument must be imported from eventNames.ts.
 * 
 * This rule enforces that the first argument to `eventBridge.on()`
 * must be a reference to a constant imported from `src/api/eventNames.ts`,
 * not a magic string literal.
 * 
 * Good:  eventBridge.on(MESSAGE_CHUNKED, handler)
 * Bad:   eventBridge.on('message/chunked', handler)
 */

'use strict';

module.exports = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'eventBridge.on() first argument must use eventNames.ts constant',
      category: 'Best Practices',
      recommended: true,
    },
    fixable: null,
    schema: [],
  },

  create(context) {
    const sourceCode = context.getSourceCode();

    function isEventBridgeOnCall(node) {
      return (
        node.type === 'CallExpression' &&
        node.callee.type === 'MemberExpression' &&
        node.callee.property.name === 'on' &&
        node.callee.object &&
        (node.callee.object.name === 'eventBridge' ||
         (node.callee.object.type === 'MemberExpression' &&
          node.callee.object.property.name === 'on'))
      );
    }

    return {
      CallExpression(node) {
        if (!isEventBridgeOnCall(node)) {
          return;
        }

        const firstArg = node.arguments[0];
        if (!firstArg) {
          return;
        }

        // Allow identifier references (e.g., MESSAGE_CHUNKED)
        if (firstArg.type === 'Identifier') {
          return;
        }

        // Allow member expressions (e.g., EventType.MessageStarted)
        if (firstArg.type === 'MemberExpression') {
          return;
        }

        // Check if it's a string literal (which is what we want to prevent)
        if (firstArg.type === 'Literal' && typeof firstArg.value === 'string') {
          context.report({
            node: firstArg,
            message: 'eventBridge.on() first argument must use a constant from src/api/eventNames.ts, not a string literal. Use {{ value }} instead.',
            data: {
              value: firstArg.value,
            },
          });
        }

        // Also catch template literals
        if (firstArg.type === 'TemplateLiteral' && firstArg.expressions.length === 0) {
          context.report({
            node: firstArg,
            message: 'eventBridge.on() first argument must use a constant from src/api/eventNames.ts.',
          });
        }
      },
    };
  },
};

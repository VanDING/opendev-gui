# Deepwork: Claude Code vs OpenDev Desktop Comparison

## Complete. 10 phases, all subsystems compared.

## Oracle Verdict

### Maturity: ~70-75% complete
OD has the harder, architecturally sound parts (agent loop, tool safety, memory, fuzzy matching). Gaps are in ecosystem integration and UX automation — "breadth" gaps, not "depth" gaps. Foundation is solid.

### Top CC Features Missing from OD
1. **MCP OAuth 2.0 + PKCE** — blocks most real-world MCP servers (critical ceiling)
2. **Auto-approve classifier** — high UX friction for multi-file edits
3. **Global prompt cache** — 5-13K tokens saved per session, significant cost
4. **MCP WebSocket transport** — emerging standard, SSE has known limits
5. **MCP elicitation** — blocks interactive MCP servers

### Top OD Features Missing from CC
1. **9-pass fuzzy edit matching** — dramatically improves edit success rates
2. **LSP post-edit diagnostics** — closed feedback loop, catches errors immediately
3. **Input-aware bash safety** — safely parallelizes read-only bash
4. **Dual bash timeout (idle + max)** — catches hanging commands
5. **SSRF protection via DNS resolution** — security feature CC lacks

### Rust Investment: Paid Off
Safety-critical areas benefit (process management, file ops, concurrency, network security). Complexity tax is real in web subsystems (MCP, HTTP, JSON). Hybrid Rust core + React UI is architecturally sound. Unfinished sandbox means strongest safety argument isn't yet realized.

### Recommended Next Steps
1. MCP OAuth 2.0 + PKCE (2-3 weeks)
2. Auto-approve heuristics (1-2 weeks)
3. Complete sandbox integration (2-3 weeks)
4. Replace DDG scraping with real API (1 week)
5. Global prompt cache (1 week)

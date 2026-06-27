/**
 * Mock AI Proxy — standalone Node.js HTTP server for Playwright E2E tests.
 *
 * Runs on port 11436 (configurable via PORT env var).
 * Implements the same POST /v1/propose contract as the real Rust ai-proxy,
 * but returns canned CommandEnvelope[] responses keyed by prompt patterns.
 *
 * Usage:
 *   node mock-ai-proxy.mjs
 *   PORT=11437 node mock-ai-proxy.mjs
 *
 * Responses are configured via the `RESPONSES` map below.
 * If a prompt matches a pattern key, the matching canned response is returned.
 * Otherwise, the DEFAULT_RESPONSE is returned.
 */

import { createServer } from "node:http";

const PORT = Number(process.env.PORT ?? 11436);

// ─── Canned responses ─────────────────────────────────────────────────────────

const COMMAND_ENVELOPES = {
  create_entity: {
    command: { type: "CreateEntity", id: "ent_ai_001", name: "AI Sprite" },
    metadata: {
      authorship: "agent:gpt-4o",
      timestamp: Date.now(),
      rationale: "Created a new sprite entity named 'AI Sprite'",
      model: "gpt-4o",
    },
  },
  set_field: {
    command: {
      type: "SetComponentField",
      entity_id: "ent_ai_001",
      type_id: "editor.Sprite2D",
      field_path: "translation.x",
      value: 100,
    },
    metadata: {
      authorship: "agent:gpt-4o",
      timestamp: Date.now(),
      rationale: "Set translation.x to 100",
      model: "gpt-4o",
    },
  },
};

/** Response map: prompt substring → ProposeResponse body */
const RESPONSES = new Map([
  [
    "create sprite",
    {
      commands: [
        COMMAND_ENVELOPES.create_entity,
        COMMAND_ENVELOPES.set_field,
      ],
      rationale: "Created a new sprite entity named 'AI Sprite'",
      model: "gpt-4o",
    },
  ],
  [
    "add enemy",
    {
      commands: [
        {
          command: { type: "CreateEntity", id: "ent_ai_002", name: "Enemy" },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created enemy entity",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created an enemy entity",
      model: "gpt-4o",
    },
  ],
  [
    "default",
    {
      commands: [
        {
          command: { type: "CreateEntity", id: "ent_ai_003", name: "Generic Entity" },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Generic suggestion",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Added a generic entity",
      model: "gpt-4o",
    },
  ],
]);

/** Look up a response by prompt substring match */
function lookupResponse(prompt) {
  const lower = prompt.toLowerCase();
  for (const [pattern, response] of RESPONSES) {
    if (pattern !== "default" && lower.includes(pattern)) {
      return response;
    }
  }
  return RESPONSES.get("default");
}

// ─── HTTP server ─────────────────────────────────────────────────────────────

function parseBody(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => (data += chunk));
    req.on("end", () => {
      try {
        resolve(JSON.parse(data));
      } catch {
        reject(new Error("Invalid JSON"));
      }
    });
    req.on("error", reject);
  });
}

const server = createServer(async (req, res) => {
  // CORS headers
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://localhost:${PORT}`);

  // POST /v1/propose
  if (req.method === "POST" && url.pathname === "/v1/propose") {
    let body;
    try {
      body = await parseBody(req);
    } catch {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Invalid JSON body" }));
      return;
    }

    const prompt = body?.prompt ?? "";

    // Simulate a small delay (50-150ms) like a real AI call
    await new Promise((r) => setTimeout(r, 50 + Math.floor(Math.random() * 100)));

    const response = lookupResponse(prompt);

    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify(response));
    return;
  }

  // GET /health
  if (req.method === "GET" && url.pathname === "/health") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ status: "ok" }));
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

server.listen(PORT, () => {
  console.log(`[mock-ai-proxy] Listening on http://localhost:${PORT}`);
  console.log(
    `[mock-ai-proxy] Prompt patterns: ${[...RESPONSES.keys()]
      .filter((k) => k !== "default")
      .join(", ")}`
  );
});

server.on("error", (err) => {
  console.error(`[mock-ai-proxy] Server error: ${err.message}`);
  process.exit(1);
});

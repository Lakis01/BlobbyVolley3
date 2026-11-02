// Blobby Volley 3 - Zero-Cost Serverless Matchmaking Registry (Cloudflare Worker)
// Deploy in 30 seconds for $0/mo:
//   npm install -g wrangler
//   wrangler deploy

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Standard CORS headers for game clients
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Content-Type': 'application/json',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // GET /rooms: List active public rooms
    if (url.pathname === '/rooms' && request.method === 'GET') {
      const rooms = [];
      if (env.ROOMS_KV) {
        const list = await env.ROOMS_KV.list({ prefix: 'room:' });
        for (const key of list.keys) {
          const val = await env.ROOMS_KV.get(key.name, 'json');
          if (val) rooms.push(val);
        }
      } else {
        // In-memory fallback if KV is not bound
        if (!globalThis.inMemoryRooms) globalThis.inMemoryRooms = new Map();
        const now = Date.now();
        for (const [id, r] of globalThis.inMemoryRooms.entries()) {
          if (now - r.updated_at < 60000) {
            rooms.push(r);
          } else {
            globalThis.inMemoryRooms.delete(id);
          }
        }
      }
      return new Response(JSON.stringify(rooms), { headers: corsHeaders });
    }

    // POST /rooms: Register / Heartbeat a lobby
    if (url.pathname === '/rooms' && request.method === 'POST') {
      try {
        const body = await request.json();
        if (!body.id || !body.endpoint) {
          return new Response(JSON.stringify({ error: 'Missing id or endpoint' }), {
            status: 400,
            headers: corsHeaders,
          });
        }

        const room = {
          id: body.id,
          name: body.name || 'Remastered Court',
          host: body.host || 'Player',
          endpoint: body.endpoint,
          arena: body.arena || 'Copacabana Sunset',
          server_type: 'Online P2P Room',
          updated_at: Date.now(),
        };

        if (env.ROOMS_KV) {
          // 60-second TTL auto-prunes inactive hosts
          await env.ROOMS_KV.put(`room:${body.id}`, JSON.stringify(room), { expirationTtl: 60 });
        } else {
          if (!globalThis.inMemoryRooms) globalThis.inMemoryRooms = new Map();
          globalThis.inMemoryRooms.set(body.id, room);
        }

        return new Response(JSON.stringify({ status: 'ok', room }), { headers: corsHeaders });
      } catch (err) {
        return new Response(JSON.stringify({ error: err.message }), { status: 500, headers: corsHeaders });
      }
    }

    // DELETE /rooms/:id: Remove room when match starts or host exits
    if (url.pathname.startsWith('/rooms/') && request.method === 'DELETE') {
      const id = url.pathname.slice('/rooms/'.length);
      if (env.ROOMS_KV && id) {
        await env.ROOMS_KV.delete(`room:${id}`);
      } else if (globalThis.inMemoryRooms) {
        globalThis.inMemoryRooms.delete(id);
      }
      return new Response(JSON.stringify({ status: 'deleted', id }), { headers: corsHeaders });
    }

    return new Response(JSON.stringify({
      service: 'Blobby Volley 3 Serverless Matchmaking API',
      status: 'operational',
      version: '3.0.0',
      routes: ['GET /rooms', 'POST /rooms', 'DELETE /rooms/:id'],
    }), { headers: corsHeaders });
  },
};

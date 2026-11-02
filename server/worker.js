// Blobby Volley 3 - Zero-Cost Serverless Matchmaking Registry & Glicko-2 Queue (Cloudflare Worker)
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

    if (!globalThis.inMemoryRooms) globalThis.inMemoryRooms = new Map();
    if (!globalThis.matchQueue) globalThis.matchQueue = new Map();
    if (!globalThis.activeMatches) globalThis.activeMatches = new Map();

    const now = Date.now();

    // Clean up stale queue entries (> 60s)
    for (const [id, entry] of globalThis.matchQueue.entries()) {
      if (now - entry.joined_at > 60000) {
        globalThis.matchQueue.delete(id);
      }
    }

    // =========================================================================
    // 1. GLICKO-2 SKILL-BASED MATCHMAKING QUEUE
    // =========================================================================

    // POST /matchmaking/queue: Join matchmaking pool with Glicko-2 rating
    if (url.pathname === '/matchmaking/queue' && request.method === 'POST') {
      try {
        const body = await request.json();
        const playerId = body.player_id;
        const name = body.name || 'Player';
        const rating = Number(body.rating) || 1500.0;
        const rd = Number(body.rd) || 350.0;
        const endpoint = body.endpoint || '';

        if (!playerId) {
          return new Response(JSON.stringify({ error: 'Missing player_id' }), { status: 400, headers: corsHeaders });
        }

        // Check if player is already matched
        if (globalThis.activeMatches.has(playerId)) {
          const match = globalThis.activeMatches.get(playerId);
          return new Response(JSON.stringify({ status: 'matched', match }), { headers: corsHeaders });
        }

        // Search the queue for an opponent of similar skill
        let bestOpponent = null;
        let minRatingDiff = Infinity;

        for (const [otherId, other] of globalThis.matchQueue.entries()) {
          if (otherId === playerId) continue;

          const diff = Math.abs(rating - other.rating);
          // Tolerance window starts at +/- 100 rating points and expands by 25 points every 5 seconds waiting
          const waitSeconds = Math.floor((now - other.joined_at) / 1000);
          const allowedDiff = 100 + Math.floor(waitSeconds / 5) * 25;

          if (diff <= allowedDiff && diff < minRatingDiff) {
            minRatingDiff = diff;
            bestOpponent = other;
          }
        }

        // Opponent found! Form a match
        if (bestOpponent) {
          globalThis.matchQueue.delete(bestOpponent.player_id);

          // Deterministic role assignment: the one with verified endpoint or waiting longer hosts
          const hostPlayer = bestOpponent.endpoint ? bestOpponent : { player_id: playerId, name, rating, rd, endpoint };
          const clientPlayer = hostPlayer.player_id === playerId ? bestOpponent : { player_id: playerId, name, rating, rd, endpoint };

          const matchForHost = {
            match_id: `match_${now}`,
            role: 'host',
            opponent_name: clientPlayer.name,
            opponent_rating: clientPlayer.rating,
            opponent_rd: clientPlayer.rd,
            connect_to: hostPlayer.endpoint,
          };

          const matchForClient = {
            match_id: `match_${now}`,
            role: 'client',
            opponent_name: hostPlayer.name,
            opponent_rating: hostPlayer.rating,
            opponent_rd: hostPlayer.rd,
            connect_to: hostPlayer.endpoint,
          };

          globalThis.activeMatches.set(hostPlayer.player_id, matchForHost);
          globalThis.activeMatches.set(clientPlayer.player_id, matchForClient);

          const myMatch = playerId === hostPlayer.player_id ? matchForHost : matchForClient;
          return new Response(JSON.stringify({ status: 'matched', match: myMatch }), { headers: corsHeaders });
        }

        // No suitable opponent yet: add to queue
        globalThis.matchQueue.set(playerId, {
          player_id: playerId,
          name,
          rating,
          rd,
          endpoint,
          joined_at: now,
        });

        return new Response(JSON.stringify({ status: 'queued', position: globalThis.matchQueue.size }), { headers: corsHeaders });
      } catch (err) {
        return new Response(JSON.stringify({ error: err.message }), { status: 500, headers: corsHeaders });
      }
    }

    // GET /matchmaking/poll: Check match status while in queue
    if (url.pathname === '/matchmaking/poll' && request.method === 'GET') {
      const playerId = url.searchParams.get('player_id');
      if (globalThis.activeMatches.has(playerId)) {
        const match = globalThis.activeMatches.get(playerId);
        // Remove from matches map after polling
        globalThis.activeMatches.delete(playerId);
        return new Response(JSON.stringify({ status: 'matched', match }), { headers: corsHeaders });
      }

      if (globalThis.matchQueue.has(playerId)) {
        const entry = globalThis.matchQueue.get(playerId);
        const waitSec = Math.floor((now - entry.joined_at) / 1000);
        return new Response(JSON.stringify({
          status: 'searching',
          elapsed_seconds: waitSec,
          current_window: 100 + Math.floor(waitSec / 5) * 25,
        }), { headers: corsHeaders });
      }

      return new Response(JSON.stringify({ status: 'idle' }), { headers: corsHeaders });
    }

    // DELETE /matchmaking/cancel: Leave queue
    if (url.pathname === '/matchmaking/cancel' && request.method === 'DELETE') {
      const playerId = url.searchParams.get('player_id');
      if (playerId) {
        globalThis.matchQueue.delete(playerId);
        globalThis.activeMatches.delete(playerId);
      }
      return new Response(JSON.stringify({ status: 'cancelled' }), { headers: corsHeaders });
    }

    // =========================================================================
    // 2. ROOM DIRECTORY (LOBBIES)
    // =========================================================================

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
          await env.ROOMS_KV.put(`room:${body.id}`, JSON.stringify(room), { expirationTtl: 60 });
        } else {
          globalThis.inMemoryRooms.set(body.id, room);
        }

        return new Response(JSON.stringify({ status: 'ok', room }), { headers: corsHeaders });
      } catch (err) {
        return new Response(JSON.stringify({ error: err.message }), { status: 500, headers: corsHeaders });
      }
    }

    // DELETE /rooms/:id: Remove room
    if (url.pathname.startsWith('/rooms/') && request.method === 'DELETE') {
      const id = url.pathname.slice('/rooms/'.length);
      if (env.ROOMS_KV && id) {
        await env.ROOMS_KV.delete(`room:${id}`);
      } else {
        globalThis.inMemoryRooms.delete(id);
      }
      return new Response(JSON.stringify({ status: 'deleted', id }), { headers: corsHeaders });
    }

    return new Response(JSON.stringify({
      service: 'Blobby Volley 3 Serverless Matchmaking API',
      status: 'operational',
      version: '3.0.0',
      routes: [
        'POST /matchmaking/queue',
        'GET /matchmaking/poll?player_id=...',
        'DELETE /matchmaking/cancel?player_id=...',
        'GET /rooms',
        'POST /rooms',
        'DELETE /rooms/:id',
      ],
    }), { headers: corsHeaders });
  },
};

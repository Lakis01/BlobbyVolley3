# Blobby Volley 3 - Zero-Cost Serverless Matchmaking Registry

This directory contains a ready-to-deploy **Cloudflare Worker** that provides free global room discovery and matchmaking for **Blobby Volley 3 (Remastered)**.

---

## Why This Architecture Costs \$0.00 / month Forever

1. **Cloudflare Workers Free Tier**:
   - Includes **100,000 requests/day** for free.
   - Global low-latency edge deployment across 300+ data centers.
2. **STUN NAT Traversal**:
   - Uses Google / Cloudflare public STUN servers for direct P2P connections without bandwidth relay costs.
3. **No Heavy Compute**:
   - The game physics (75 Hz) run directly on the players' machines (authoritative host P2P). The worker only tracks small JSON room tickets (e.g. "Room ID 123 is at 93.199.209.17:1234").

---

## 30-Second Deployment

If you want your own branded matchmaking URL:

1. **Install Wrangler** (Cloudflare CLI):
   ```bash
   npm install -g wrangler
   ```

2. **Login to your free Cloudflare account**:
   ```bash
   wrangler login
   ```

3. **Deploy**:
   ```bash
   cd server
   wrangler deploy
   ```

You will get an instant public URL, for example:
`https://blobby-volley-3-matchmaking.<your-username>.workers.dev`

You can then set this URL in `~/.config/blobbyvolley3/config.json`:
```json
{
  "matchmaking_registry_url": "https://blobby-volley-3-matchmaking.<your-username>.workers.dev"
}
```
Or leave it blank to rely on auto-discovered local LAN rooms and direct IP connections!

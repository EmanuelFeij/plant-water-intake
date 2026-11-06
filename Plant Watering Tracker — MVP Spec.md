# Plant Watering Tracker — MVP Spec

Nov 4, 2026 · @Emanuel Feijó

## Goal and scope

Tap an NFC sticker on a plant pot to log a watering, and get one daily reminder for plants that are overdue.

**In scope**

- Logging a watering with a tap on the plant's sticker
- A list page of all plants, most overdue first
- A daily push reminder
- Only my phone can log waterings

**Out of scope for now**

- Photos, watering history, growth timelapse
- A UI for adding or editing plants
- Learned watering intervals

## How it works

&#91;embedded content: tap flow and daily reminder\]

A tap only saves a timestamp when the phone has the pairing cookie. Once a day, a job reads the same table and sends one push if anything is overdue.

## Stack

One Rust binary, built on the laptop and copied to the server.

| Need | Crate | Why |
| --- | --- | --- |
| Async runtime | `tokio` | Runs the web server and the daily reminder task |
| Web server | `axum` | Small, popular, built on tokio |
| Signed cookie | `axum-extra` with the `cookie-signed` feature | `SignedCookieJar` handles the HMAC signing for the pairing cookie |
| SQLite | `rusqlite` with the `bundled` feature | Compiles SQLite in, so nothing to install on the server |
| HTML pages | `maud` | Templates checked at compile time, no template files |
| ntfy push | `reqwest` with `rustls-tls` | Avoids depending on OpenSSL |
| Time | `jiff` | Handles Europe/Amsterdam for the reminder time |

For this size of app, `rusqlite` with a `Mutex<Connection>` is enough. An async pool like `sqlx` can come later if needed.

## Data model

One SQLite table is enough. History can come later.

**plants**

| Column | Type | Notes |
| --- | --- | --- |
| slug | text, primary key | Stable ID used in the tag URL, e.g. `monstera`, `pothos-kitchen`. Never changes. |
| name | text | Display name, free to rename |
| interval\_days | integer | How often it needs water |
| last\_watered\_at | timestamp, nullable | Null until the first tap |

Plants are added by hand with SQL. Overdue means `last_watered_at + interval_days < now`, or never watered.

## Endpoints

Three endpoints, all GET, because the phone opens tag URLs in the browser.

| Endpoint | Needs cookie | Does |
| --- | --- | --- |
| `GET /w/{slug}` | Yes | Sets `last_watered_at = now`, shows a small "Monstera watered ✓, last was 6 days ago" page |
| `GET /` | Yes | Lists plants, most overdue first, with days since last watering |
| `GET /pair?code=…` | No, needs the pairing code | Sets the auth cookie on this phone |

Rules for `/w/{slug}`:

- Unknown slug → 404
- A repeat tap within 10 minutes does not move the timestamp, but still shows the success page
- No cookie → 401 with a short "not allowed" page

## Auth

A long-lived cookie on my phone is the only thing that allows logging. Tag URLs are not secret.

1. A `PAIRING_CODE` env var holds a long random string.
2. I open `/pair?code=<PAIRING_CODE>` once on my phone, in the default browser.
3. The server sets a signed cookie: HttpOnly, Secure, SameSite=Lax, about 1 year.
4. Every request to `/` and `/w/{slug}` checks the signature.

The cookie is signed with an HMAC key from a `COOKIE_SECRET` env var. Rotating that secret logs every device out.

Open question: should pairing work only once, or can I pair a second device later?

## Reminders

One push a day, and only when something is overdue.

- A job runs daily at a fixed morning time, Europe/Amsterdam.
- It selects overdue plants.
- If there are any, it sends one ntfy.sh message, for example "Water today: Monstera, Pothos (kitchen)", with a link to `/`.
- If none are overdue, it sends nothing.
- The ntfy topic name is a random string kept in an env var, because ntfy topics are public by name.

The job is a `tokio::spawn` task in the same binary. It sleeps until the next reminder time, runs the check, and repeats. A separate cron is only needed if the host sleeps.

## NFC tags

Each sticker holds one NDEF URL record, `https://<host>/w/<slug>`.

- Stickers: NTAG213, about 144 bytes, which is plenty for a short URL.
- Write them with NFC Tools on Android.
- Test each tag with a tap before locking it.
- Lock it read-only afterwards so nobody can overwrite it. Locking is permanent.
- Stick it on the side of the pot, away from soil and water. Avoid metal pots, they block the signal.

Open question: which host or domain the URLs use. Changing it later means rewriting every tag, so pick it before writing.

## Deployment and setup

One Rust binary with SQLite on a persistent disk, behind HTTPS.

- **Host:** a small Hetzner VPS running Ubuntu LTS, set up by hand to learn the platform side.
- **Stack:** Caddy in front for HTTPS (automatic Let's Encrypt), proxying to the app on `localhost:8080`, run as a systemd service under its own user.
- **Firewall:** `ufw` allowing only 22, 80 and 443. Allow SSH before running `ufw enable`.
- **Build:** `cargo zigbuild --release --target x86_64-unknown-linux-musl` on the laptop gives one static binary, which also handles the C code in bundled SQLite. Then `scp` it over. No Rust toolchain on the server.
- **Config:** env vars `PAIRING_CODE`, `COOKIE_SECRET`, `NTFY_TOPIC`, `REMINDER_TIME`, `DB_PATH`, in an env file with mode 600.
- **Logs:** `tracing` in the app, read with `journalctl -u <service> -f`.
- **Backup:** a cron job copying the SQLite file off the server. Losing it only loses timestamps.

Setup order:

1. Deploy and check that `/` returns 401.
2. Pair the phone.
3. Insert the plants with SQL.
4. Write, test and lock the tags.
5. Subscribe to the ntfy topic on the phone.

**Guides**

1. Server setup (SSH keys, non-root user, ufw, Fail2Ban): [Setting Up Your Own Linux Server with Hetzner](https://dev.to/itpraktika/setting-up-your-own-linux-server-with-hetzner-a-technical-guide-34am). Hetzner's own [initial setup guide](https://community.hetzner.com/tutorials/howto-initial-setup-ubuntu/de/) (German version) is an alternative.
2. Running the Go app (systemd unit, Caddy, HTTPS): [Running Go Applications on a VPS](https://domainindia.com/support/kb/running-go-applications-vps-systemd-reverse-proxy). Skip the nginx part and use its Caddy section.

## Done when

- [ ] Tapping a tag on my phone logs the watering in under 2 seconds
- [ ] Tapping from another phone gets "not allowed"
- [ ] `/` shows overdue plants at the top
- [ ] An overdue plant triggers exactly one push the next morning
- [ ] Used for 2 weeks without touching the database by hand, apart from adding plants

**Later, once the MVP has proven useful:** watering history, photos per plant and a timelapse, a dashboard with colour-coded cards, an add-plant form, and intervals learned from history.

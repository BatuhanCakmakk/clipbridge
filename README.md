<p align="center"><img src="assets/tray.svg" width="72" alt=""></p>

<h1 align="center">ClipBridge</h1>

<p align="center">Copy on your iPhone, paste on Windows. And back.<br>
No app on the phone, no cloud, no account.</p>

<p align="center"><b>English</b> · <a href="README.tr.md">Türkçe</a></p>

---

ClipBridge is a small Windows tray app (a single ~4 MB exe). On the iPhone there is no App Store app: it uses Apple's own **Shortcuts**. Double-tap the back of the phone and the iPhone clipboard goes to the PC; triple-tap and the PC clipboard comes to the iPhone. Everything stays on your home network.

## Features

- **Text, images and files**, both ways. Several files from the PC arrive on the iPhone as a ZIP.
- **iPhone photos (HEIC)** become pasteable images on Windows. Photos are saved as JPG, screenshots as lossless PNG.
- **Rich text** copied in Notes, Safari or Mail pastes into Word with its formatting.
- **Links** open in the PC's default browser (can be turned off in the tray menu).
- **Stable network name**: the PC announces itself as `clipbridge-<computer-name>.local`, so the shortcuts keep working when the router hands out a new IP address.
- **QR setup**: no address or token to type.
- **Update notice** in the tray menu when a new version is out.
- **English and Turkish**: the PC follows the Windows display language; messages on the phone follow the phone's language.
- Uses no CPU while idle. It doesn't watch the Windows clipboard; it only reads it when the phone asks.

## Setup

### 1. Windows

1. Download `ClipBridge.exe` from [Releases](https://github.com/BatuhanCakmakk/clipbridge/releases/latest) and put it in a permanent folder (e.g. `Documents\ClipBridge`). Its settings are kept in a `ClipBridge-data` folder next to it.
2. Run it. If SmartScreen warns, choose **More info → Run anyway** (the exe isn't code-signed).
3. If Windows Firewall asks, allow **Private networks**.
4. The setup page opens in the browser on first start. To open it later: right-click the tray icon → **Setup / connection info (QR)**.
5. Optional: tray menu → **Start with Windows**.

### 2. iPhone

1. Add the three shortcuts:
   - [ClipBridge Setup](ICLOUD_SETUP)
   - [Send to PC](ICLOUD_SEND)
   - [Get from PC](ICLOUD_GET)
2. Run **ClipBridge Setup** and point the camera at the QR code on the PC screen.
3. Settings → Accessibility → Touch → **Back Tap**:
   - **Double Tap** → Send to PC
   - **Triple Tap** → Get from PC
4. The first time, iOS asks for local network access: **Allow**.

To build the shortcuts yourself: [docs/SHORTCUTS.md](docs/SHORTCUTS.md)

## Usage

| To | Do |
|---|---|
| iPhone → PC | Copy on the iPhone, **double-tap** the back of the phone, press **Ctrl+V** on the PC |
| Send a photo or file | In Photos or Files: **Share → Send to PC** |
| PC → iPhone | Copy on the PC, **triple-tap** the back of the phone, paste on the iPhone |

Files from the iPhone are saved in `Downloads\ClipBridge`. That folder is not cleaned up automatically.

## Troubleshooting

| Symptom | Fix |
|---|---|
| The shortcut can't connect to the server | Are the iPhone and the PC on the same network? The Windows network profile must be **Private** (Settings → Network & internet → your network → Network profile type), and ClipBridge must be allowed through the firewall on private networks. |
| The `.local` name doesn't work (some routers block mDNS) | Use the IP address shown on the setup page: set `url` in `ClipBridge.txt` to `http://192.168.x.x:8787`. |
| "Wrong token" | Scan the QR code again. The token is in `ClipBridge-data\config.json`. |
| "File not found" at the start of a shortcut | **ClipBridge Setup** hasn't been run yet. |
| To see what happened | Tray menu → **Open log** |

## Security

- ClipBridge serves only your local network and requires a 20-character token on every request. It opens nothing to the internet; don't forward its port on your router.
- The connection is plain HTTP, so traffic on the network is not encrypted. Don't use it on networks you don't trust (cafés, hotels, campus Wi-Fi).
- The setup page can only be opened on the PC itself. The QR code contains the token, so don't share a screenshot of it.
- To get a new token, delete the `token` value in `config.json`, restart ClipBridge and scan the QR code again.

## Settings (`ClipBridge-data\config.json`)

| Field | Default | Meaning |
|---|---|---|
| `port` | `8787` | Port to listen on |
| `token` | random | The `X-Token` the shortcuts send |
| `hostname` | `clipbridge-<pc-name>` | Name announced on the network (`.local` is added) |
| `language` | `auto` | `auto` (Windows display language), `en` or `tr` |
| `notifications` | `true` | Windows notification for received content |
| `open_links` | `true` | Open a single received link in the browser |
| `update_check` | `true` | Check GitHub for a new version once a day |

Restart ClipBridge after a change.

## API

To use your own tool instead of the shortcuts:

| Request | Description |
|---|---|
| `GET /ping` | Returns `ok`. For testing the address and token. |
| `POST /clip` | The body goes to the PC clipboard. Raw body or `multipart/form-data` (field name `file`). An `X-Filename` header (URL-encoded) names files. |
| `GET /clip` | The PC clipboard: text, PNG, a single file, or a ZIP for several files. `204` when empty. |

All of them need the `X-Token` header. Replies are in the language of the `Accept-Language` header.

```bash
curl -H "X-Token: <token>" --data-binary "hello" http://clipbridge-pc.local:8787/clip
```

## Translations

All text is in [`src/i18n.rs`](src/i18n.rs). Adding a language means adding one table there; pull requests are welcome.

## Building

Needs Rust (stable, MSVC):

```bash
cargo build --release
```

Output: `target/release/clipbridge.exe`. Pushing a `v*` tag makes GitHub Actions build the exe and publish it on Releases.

## License

[MIT](LICENSE). Tray icon: [Iconoir](https://iconoir.com) "send-diagonal" (MIT).

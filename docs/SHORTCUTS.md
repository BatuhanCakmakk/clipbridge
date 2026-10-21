# Building the shortcuts by hand

*[Türkçe](KESTIRMELER.md)*

Installing from the iCloud links in the README is enough for most people. This page is for building the shortcuts from scratch, or for seeing how they work.

There are three shortcuts. Apple's own action names (such as "Get Contents of URL") appear in each phone's language, so the same shortcuts work everywhere.

| Shortcut | What it does | Suggested trigger |
|---|---|---|
| **ClipBridge Setup** | Scans the QR code on the PC and saves the address and token on the iPhone. Run once. | — |
| **Send to PC** | Sends the iPhone clipboard (or shared items) to the PC. | Back Tap ×2, Share sheet |
| **Get from PC** | Puts the PC clipboard on the iPhone clipboard. | Back Tap ×3 |

The settings live in iCloud Drive at `Shortcuts/ClipBridge.txt`. The shortcuts themselves contain no address or token, which is why they can be shared.

---

## 1. ClipBridge Setup

| # | Action | Settings |
|---|---|---|
| 1 | **Scan QR/Barcode** | — |
| 2 | **Save File** | Input: **QR/Barcode** · **Ask Where to Save**: off · **Subpath**: `ClipBridge.txt` · **Overwrite If File Exists**: on |
| 3 | **Show Notification** | `ClipBridge is ready` |

The file goes to iCloud Drive → **Shortcuts** by default.

## 2. At the top of both other shortcuts: read the settings

"Send to PC" and "Get from PC" start with these actions:

| # | Action | Settings |
|---|---|---|
| 1 | **Get File** | Folder: **Shortcuts** · Path: `ClipBridge.txt` · **Error If Not Found**: on |
| 2 | **Get Dictionary from Input** | Input: **File** |
| 3 | **Get Dictionary Value** | Key: `url` |
| 4 | **Set Variable** | Name: `Address` |
| 5 | **Get Dictionary Value** | Key: `token`. Tap its input and pick **Dictionary** from step 2; it is filled in wrongly by default. |
| 6 | **Set Variable** | Name: `Token` |

If the file is missing, the shortcut stops with an error: "ClipBridge Setup" has not been run yet.

## 3. Send to PC

| # | Action | Settings |
|---|---|---|
| 0 | **ⓘ** → **Show in Share Sheet** | On. Input types: Images, Files, Text, Rich Text, URLs, Media |
| 0 | "If there's no input" | **Get Clipboard** |
| 1–6 | Read the settings (above) | |
| 7 | **Repeat with Each** | Items: **Shortcut Input** |
| 8 | Inside the loop: **Get Contents of URL** | URL: `Address` variable + `/clip` · Method: **POST** · Header: `X-Token` = `Token` variable · Request Body: **Form** → type **File**, key `file`, value **Repeat Item** |
| 9 | After the loop: **Show Notification** (optional) | `Sent to PC` |

```
Get File  Shortcuts/ClipBridge.txt
Get Dictionary from Input
Get Dictionary Value "url"    →  Set Variable Address
Get Dictionary Value "token"  →  Set Variable Token
Repeat with Each item in Shortcut Input
    Get Contents of URL  [Address]/clip
        Method: POST · Headers: X-Token = [Token]
        Request Body: Form → file (File) = Repeat Item
End Repeat
```

In the form, `file` goes in the **key** field and **Repeat Item** in the **value** field. Swapped, the PC answers "No file field in the form".

## 4. Get from PC

| # | Action | Settings |
|---|---|---|
| 1–6 | Read the settings (above) | |
| 7 | **Get Contents of URL** | URL: `[Address]/clip` · Method: **GET** · Header: `X-Token` = `[Token]` |
| 8 | **Copy to Clipboard** | Input: **Contents of URL** |
| 9 | **Show Notification** (optional) | `Copied from PC` |

## 5. Triggers

Settings → Accessibility → Touch → **Back Tap**:
- **Double Tap** → Send to PC
- **Triple Tap** → Get from PC

On iPhone 15 Pro and later, the **Action Button** works too.

## 6. Sharing the shortcuts

In the Shortcuts app, long-press a shortcut → **Share** → **Copy iCloud Link**. The shortcuts hold no secrets, so the link can go to anyone.

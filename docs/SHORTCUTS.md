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
| 1 | **Get File from Folder** | Folder: **Shortcuts** · Path: `ClipBridge.txt` · **Error If Not Found**: on |
| 2 | **Get Dictionary Value** | Input: **File** · Key: `url` (the JSON file is turned into a dictionary automatically) |
| 3 | **Set Variable** | Name: `Address` |
| 4 | **Get Dictionary Value** | Key: `token`. Its input is filled in wrongly by default: long-press the input → **Select Variable** → **File** from step 1. |
| 5 | **Set Variable** | Name: `Token` |

If the file is missing, the shortcut stops with an error: "ClipBridge Setup" has not been run yet.

Tip: to put a variable into a field, **long-press** the field → **Select Variable**. A short tap often opens the file picker or the keyboard instead.

## 3. Send to PC

Links take a separate branch: iOS can't put a link into a form's **File** field ("WFURLContentItem produced no file representation"), so a link is sent in a **Text** field.

| # | Action | Settings |
|---|---|---|
| 0 | **ⓘ** → **Show in Share Sheet** | On. Input types: all |
| 0 | "If there's no input" | **Get Clipboard** |
| 1–5 | Read the settings (above) | |
| 6 | **Repeat with Each** | Items: **Shortcut Input** |
| 7 | **Text** | Content: only **Repeat Item** |
| 8 | **If** | **Text** **begins with** `http` |
| 9 | **Get Contents of URL** | URL: `[Address]/clip` · **POST** · Header `X-Token` = `[Token]` · **Form** → type **Text**, key `file`, value **Repeat Item** |
| 10 | **Otherwise** | |
| 11 | **Get Contents of URL** | Same as 9, but the form field's type is **File** |
| 12 | **End If**, **End Repeat** | |

```
Get File from Folder  Shortcuts/ClipBridge.txt
Get Value for "url" in File    →  Set Variable Address
Get Value for "token" in File  →  Set Variable Token
Repeat with Each item in Shortcut Input
    Text: [Repeat Item]
    If Text begins with http
        Get Contents of URL [Address]/clip   POST · X-Token = [Token] · Form: file (Text) = Repeat Item
    Otherwise
        Get Contents of URL [Address]/clip   POST · X-Token = [Token] · Form: file (File) = Repeat Item
    End If
End Repeat
```

In the form, `file` goes in the **key** field and **Repeat Item** in the **value** field. Swapped, the PC answers "No file field in the form". The same goes for the header: key `X-Token`, value **Token**.

## 4. Get from PC

| # | Action | Settings |
|---|---|---|
| 1–5 | Read the settings (above) | |
| 6 | **Get Contents of URL** | URL: `[Address]/clip` · Method: **GET** · Header: `X-Token` = `[Token]` |
| 7 | **Copy to Clipboard** | Input: **Contents of URL** |
| 8 | **Show Notification** (optional) | `Copied from PC` |

## 5. Triggers

Settings → Accessibility → Touch → **Back Tap**:
- **Double Tap** → Send to PC
- **Triple Tap** → Get from PC

On iPhone 15 Pro and later, the **Action Button** works too.

## 6. Sharing the shortcuts

In the Shortcuts app, long-press a shortcut → **Share** → **Copy iCloud Link**. The shortcuts hold no secrets, so the link can go to anyone.

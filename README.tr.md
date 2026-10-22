<p align="center"><img src="assets/tray.svg" width="72" alt=""></p>

<h1 align="center">ClipBridge</h1>

<p align="center">iPhone'da kopyala, Windows'ta yapıştır. Tersi de.<br>
Telefonda uygulama yok, bulut yok, hesap yok.</p>

<p align="center"><a href="README.md">English</a> · <b>Türkçe</b></p>

<p align="center">

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20me%20a%20coffee-FFDD00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/batuhancakmak)

</p>

---

ClipBridge, Windows'ta tepside çalışan küçük bir programdır (tek exe, ~4 MB). iPhone tarafında App Store uygulaması yok; Apple'ın kendi **Kestirmeler** uygulaması kullanılır. Telefonun arkasına iki kez dokununca iPhone panosu PC'ye gider, üç kez dokununca PC panosu iPhone'a gelir. Her şey ev ağında kalır.

## Özellikler

- **Metin, resim, dosya**: iki yönde de. Birden çok dosya PC'den iPhone'a ZIP olarak gelir.
- **iPhone fotoğrafları (HEIC)** Windows'ta yapıştırılabilir resme çevrilir. Fotoğraflar JPG, ekran görüntüleri kayıpsız PNG olarak kaydedilir.
- **Biçimli metin**: Notlar, Safari ya da Mail'den kopyalanan metin Word'e biçimiyle yapıştırılır.
- **Linkler** PC'de varsayılan tarayıcıda açılır (tepsi menüsünden kapatılabilir).
- **Sabit ağ adı**: PC ağda `clipbridge-<bilgisayar-adı>.local` adıyla duyurulur. Modem IP adresini değiştirse de kestirmeler çalışmaya devam eder.
- **QR ile kurulum**: Adres ya da token yazmak gerekmez.
- **Güncelleme bildirimi**: Yeni sürüm çıkınca tepsi menüsünde görünür.
- **Türkçe ve İngilizce**: PC tarafı Windows'un diline, telefondaki mesajlar telefonun diline uyar.
- Boştayken işlemci kullanmaz. Windows panosunu izlemez; yalnızca telefon isteyince okur.

## Kurulum

### 1. Windows

1. [Releases](https://github.com/BatuhanCakmakk/clipbridge/releases/latest) sayfasından `ClipBridge.exe` dosyasını indir ve kalıcı bir klasöre koy (ör. `Belgeler\ClipBridge`). Ayarlar exe'nin yanındaki `ClipBridge-data` klasöründe tutulur.
2. Çalıştır. SmartScreen uyarı verirse **Ek bilgi → Yine de çalıştır** (exe imzalı değil).
3. Güvenlik duvarı sorarsa **Özel ağlar** için **İzin ver**.
4. İlk açılışta kurulum sayfası tarayıcıda açılır. Sonradan açmak için: tepsi simgesine sağ tıkla → **Kurulum / bağlantı bilgileri (QR)**.
5. İsteğe bağlı: tepsi menüsü → **Windows ile başlat**.

### 2. iPhone

1. Üç kestirmeyi ekle:
   - [ClipBridge Setup](https://www.icloud.com/shortcuts/0bacffec409047adb8eabd7d4bb0a65c)
   - [Send to PC](https://www.icloud.com/shortcuts/12564ab439004261af1ad149a6d74f32)
   - [Get from PC](https://www.icloud.com/shortcuts/e10c7003c8b44877b02c01c309e0ad22)
2. **ClipBridge Setup** kestirmesini çalıştır ve kamerayı PC ekranındaki QR koda tut.
3. Ayarlar → Erişilebilirlik → Dokunma → **Arkaya Dokun**:
   - **Çift Dokunma** → Send to PC
   - **Üçlü Dokunma** → Get from PC
4. İlk seferde iOS yerel ağ izni ister: **İzin Ver**.
5. İsteğe bağlı ama önerilir: Dosyalar uygulamasında `iCloud Drive/Shortcuts/ClipBridge.txt` dosyasına uzun bas → **İndirilmiş Öğeyi Tut**. Kestirmeler dosyayı iCloud yerine telefondan okur; daha hızlıdır ve internet kesikken de çalışır.

Kestirmeleri kendin kurmak istersen: [docs/KESTIRMELER.md](docs/KESTIRMELER.md)

## Kullanım

| Yapmak istediğin | Nasıl |
|---|---|
| iPhone → PC | iPhone'da kopyala, telefonun arkasına **iki kez** dokun, PC'de **Ctrl+V** |
| Fotoğraf / dosya göndermek | Fotoğraflar ya da Dosyalar'da **Paylaş → Send to PC** |
| PC → iPhone | PC'de kopyala, telefonun arkasına **üç kez** dokun, iPhone'da yapıştır |

iPhone'dan gelen dosyalar `İndirilenler\ClipBridge` klasörüne kaydedilir. Bu klasör kendiliğinden temizlenmez.

## Sorun giderme

| Belirti | Çözüm |
|---|---|
| Kestirme sunucuya bağlanamıyor | iPhone ve PC aynı ağda mı? Windows'ta ağ profili **Özel** olmalı (Ayarlar → Ağ ve İnternet → ağın → Ağ profili türü). Güvenlik duvarında ClipBridge'e Özel ağ izni verilmiş olmalı. |
| `.local` adı çalışmıyor (bazı modemler mDNS'i engeller) | Kurulum sayfasındaki IP adresini kullan: `ClipBridge.txt` dosyasındaki `url` değerini `http://192.168.x.x:8787` yap. |
| "Token hatalı" | QR kodu yeniden okut. Token `ClipBridge-data\config.json` dosyasında. |
| Kestirmenin başında "Dosya bulunamadı" | **ClipBridge Setup** henüz çalıştırılmamış. |
| Ne olduğunu görmek | Tepsi menüsü → **Günlüğü aç** |

## Güvenlik

- ClipBridge yalnızca yerel ağa hizmet verir ve her istekte 20 karakterlik bir token ister. İnternete kapı açmaz; modemde port yönlendirme yapma.
- Bağlantı düz HTTP'dir, ağdaki trafik şifrelenmez. Güvenmediğin ağlarda (kafe, otel, okul) kullanma.
- Kurulum sayfası yalnızca PC'nin kendisinden açılabilir. QR kodda token olduğu için ekran görüntüsünü paylaşma.
- Token'ı yenilemek için `config.json` dosyasındaki `token` değerini sil, ClipBridge'i yeniden başlat ve QR kodu tekrar okut.

## Ayarlar (`ClipBridge-data\config.json`)

| Alan | Varsayılan | Açıklama |
|---|---|---|
| `port` | `8787` | Dinlenen port |
| `token` | rastgele | Kestirmelerin gönderdiği `X-Token` |
| `hostname` | `clipbridge-<pc-adı>` | Ağda duyurulan ad (`.local` eklenir) |
| `language` | `auto` | `auto` (Windows'un dili), `en` ya da `tr` |
| `notifications` | `true` | Gelen içerik için Windows bildirimi |
| `open_links` | `true` | Tek bir link gelince tarayıcıda aç |
| `update_check` | `true` | Günde bir kez GitHub'da yeni sürüm denetle |

Değişiklikten sonra ClipBridge'i yeniden başlat.

## API

Kestirme yerine kendi aracını yazmak istersen:

| İstek | Açıklama |
|---|---|
| `GET /ping` | `ok` döner. Adres ve token'ı denemek için. |
| `POST /clip` | Gövde PC panosuna gider. Ham gövde ya da `multipart/form-data` (alan adı `file`). Dosya adı için `X-Filename` başlığı (URL kodlu) kullanılabilir. |
| `GET /clip` | PC panosu: metin, PNG, tek dosya ya da birden çok dosya için ZIP. Pano boşsa `204`. |

Hepsi `X-Token` başlığı ister. Yanıtlar `Accept-Language` başlığındaki dilde döner.

```bash
curl -H "X-Token: <token>" --data-binary "merhaba" http://clipbridge-pc.local:8787/clip
```

## Çeviriler

Bütün metinler [`src/i18n.rs`](src/i18n.rs) dosyasında. Yeni bir dil eklemek oraya bir tablo eklemekten ibaret; katkılar memnuniyetle karşılanır.

## Derleme

Rust (stable, MSVC) gerekir:

```bash
cargo build --release
```

Çıktı: `target/release/clipbridge.exe`. Bir `v*` etiketi gönderildiğinde GitHub Actions exe'yi derleyip Releases'a koyar.

## Destek

ClipBridge işine yaradıysa bana [bir kahve ısmarlayabilirsin](https://buymeacoffee.com/batuhancakmak) ☕. Hata bildirimleri ve katkılar da en az o kadar değerli.

## Lisans

[MIT](LICENSE). Tepsi simgesi: [Iconoir](https://iconoir.com) "send-diagonal" (MIT).

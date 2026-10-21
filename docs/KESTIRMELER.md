# Kestirmeleri elle kurmak

Normalde README'deki iCloud bağlantılarından kurmak yeterli. Bu sayfa kestirmeleri sıfırdan kurmak ya da nasıl çalıştıklarını görmek isteyenler için.

Üç kestirme var:

| Kestirme | Ne yapar | Önerilen tetikleyici |
|---|---|---|
| **ClipBridge Kurulum** | PC'deki QR kodu okur, adresi ve token'ı iPhone'a kaydeder. Bir kez çalıştırılır. | — |
| **PC'ye Gönder** | iPhone panosunu (ya da paylaşılan öğeleri) PC'ye gönderir. | Arkaya Dokun ×2, Paylaş menüsü |
| **PC'den Al** | PC panosunu iPhone panosuna alır. | Arkaya Dokun ×3 |

Ayarlar iCloud Drive'da `Kestirmeler/ClipBridge.txt` dosyasında durur. Kestirmelerin içinde adres ya da token yoktur, bu yüzden paylaşılabilirler.

Eylem adları Türkçe arayüze göre, parantez içinde İngilizceleri var.

---

## 1. ClipBridge Kurulum

| # | Eylem | Ayar |
|---|---|---|
| 1 | **QR/Barkod Kodunu Tara** *(Scan QR/Barcode)* | — |
| 2 | **Dosyayı Kaydet** *(Save File)* | Girdi: **QR/Barkod Kodu** · **Nereye Kaydedileceğini Sor**: kapalı · **Alt Yol** *(Subpath)*: `ClipBridge.txt` · **Üzerine Yaz** *(Overwrite If File Exists)*: açık |
| 3 | **Bildirim Göster** *(Show Notification)* | `ClipBridge kuruldu` |

Kaydedilen klasör varsayılan olarak iCloud Drive → **Kestirmeler** *(Shortcuts)*.

## 2. Her iki kestirmenin başına: ayarları okuma

"PC'ye Gönder" ve "PC'den Al" bu beş eylemle başlar:

| # | Eylem | Ayar |
|---|---|---|
| 1 | **Dosya Al** *(Get File)* | Klasör: **Kestirmeler** · Yol: `ClipBridge.txt` · **Bulunamazsa Hata Ver**: açık |
| 2 | **Girdiden Sözlük Al** *(Get Dictionary from Input)* | Girdi: **Dosya** |
| 3 | **Sözlük Değerini Al** *(Get Dictionary Value)* | Anahtar: `adres` |
| 4 | **Değişkeni Ayarla** *(Set Variable)* | Ad: `Adres` |
| 5 | **Sözlük Değerini Al** + **Değişkeni Ayarla** | Anahtar: `token` → Ad: `Token` (sözlük girdisi olarak 2. adımdaki **Sözlük**'ü seç) |

Dosya bulunamazsa kestirme hata verir. Bu, "ClipBridge Kurulum" kestirmesinin henüz çalıştırılmadığı anlamına gelir.

## 3. PC'ye Gönder

| # | Eylem | Ayar |
|---|---|---|
| 0 | **ⓘ** → **Paylaşma Sayfasında Göster** | Açık. Girdi türleri: Görüntüler, Dosyalar, Metin, Zengin Metin, URL'ler, Medya |
| 0 | "Girdi yoksa" | **Panoyu Al** *(Get Clipboard)* |
| 1–5 | Ayarları okuma (yukarıda) | |
| 6 | **Her Biriyle Yinele** *(Repeat with Each)* | Öğe: **Kestirme Girdisi** *(Shortcut Input)* |
| 7 | Yinelemenin içinde: **URL'nin İçeriğini Al** *(Get Contents of URL)* | URL: `Adres` değişkeni + `/clip` · Yöntem: **POST** · Başlık: `X-Token` = `Token` değişkeni · İstek Gövdesi: **Form** → Tür **Dosya**, Anahtar `file`, Değer **Yinelenen Öğe** |
| 8 | Yinelemenin dışında: **Bildirim Göster** (isteğe bağlı) | `PC'ye gönderildi` |

```
Dosya Al  Kestirmeler/ClipBridge.txt
Girdiden Sözlük Al
Sözlük Değerini Al "adres"  →  Değişkeni Ayarla Adres
Sözlük Değerini Al "token"  →  Değişkeni Ayarla Token
Kestirme Girdisi'ndeki her öğe ile yinele
    URL'nin İçeriğini Al  [Adres]/clip
        Yöntem: POST · Başlıklar: X-Token = [Token]
        İstek Gövdesi: Form → file (Dosya) = Yinelenen Öğe
Yinelemeyi Bitir
```

Form'da dikkat: `file` **anahtar** alanına, **Yinelenen Öğe** ise **değer** alanına girer. İkisi yer değiştirirse PC "formda dosya alanı bulunamadı" der.

## 4. PC'den Al

| # | Eylem | Ayar |
|---|---|---|
| 1–5 | Ayarları okuma (yukarıda) | |
| 6 | **URL'nin İçeriğini Al** | URL: `[Adres]/clip` · Yöntem: **GET** · Başlık: `X-Token` = `[Token]` |
| 7 | **Panoya Kopyala** *(Copy to Clipboard)* | Girdi: **URL'nin İçeriği** |
| 8 | **Bildirim Göster** (isteğe bağlı) | `Panoya kopyalandı` |

## 5. Tetikleyiciler

Ayarlar → Erişilebilirlik → Dokunma → **Arkaya Dokun**:
- **Çift Dokunma** → PC'ye Gönder
- **Üçlü Dokunma** → PC'den Al

iPhone 15 Pro ve sonrasında **Eylem Düğmesi** de kullanılabilir.

## 6. Kestirmeleri paylaşmak

Kestirmeler uygulamasında kestirmeye uzun bas → **Paylaş** → **iCloud Bağlantısını Kopyala**. Kestirmelerde gizli bilgi olmadığı için bağlantı herkese verilebilir.

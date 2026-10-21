# Kestirmeleri elle kurmak

*[English](SHORTCUTS.md)*

Normalde README'deki iCloud bağlantılarından kurmak yeterli. Bu sayfa kestirmeleri sıfırdan kurmak ya da nasıl çalıştıklarını görmek isteyenler için.

Üç kestirme var. Adları İngilizce, çünkü aynı kestirmeler her dildeki iPhone'da kullanılıyor. Apple'ın eylem adları (ör. "URL'nin İçeriğini Al") her telefonda o telefonun dilinde görünür.

| Kestirme | Ne yapar | Önerilen tetikleyici |
|---|---|---|
| **ClipBridge Setup** | PC'deki QR kodu okur, adresi ve token'ı iPhone'a kaydeder. Bir kez çalıştırılır. | — |
| **Send to PC** | iPhone panosunu (ya da paylaşılan öğeleri) PC'ye gönderir. | Arkaya Dokun ×2, Paylaş menüsü |
| **Get from PC** | PC panosunu iPhone panosuna alır. | Arkaya Dokun ×3 |

Ayarlar iCloud Drive'da `Kestirmeler/ClipBridge.txt` dosyasında durur. Kestirmelerin içinde adres ya da token yoktur, bu yüzden paylaşılabilirler.

Eylem adları Türkçe arayüze göre yazıldı, parantez içinde İngilizceleri var.

---

## 1. ClipBridge Setup

| # | Eylem | Ayar |
|---|---|---|
| 1 | **QR/Barkod Kodunu Tara** *(Scan QR/Barcode)* | — |
| 2 | **Dosyayı Kaydet** *(Save File)* | Girdi: **QR/Barkod Kodu** · **Nereye Kaydedileceğini Sor**: kapalı · **Alt Yol** *(Subpath)*: `ClipBridge.txt` · **Üzerine Yaz** *(Overwrite If File Exists)*: açık |
| 3 | **Bildirim Göster** *(Show Notification)* | `ClipBridge is ready` |

Dosya varsayılan olarak iCloud Drive → **Kestirmeler** *(Shortcuts)* klasörüne kaydedilir.

## 2. İki kestirmenin başına: ayarları okuma

"Send to PC" ve "Get from PC" şu eylemlerle başlar:

| # | Eylem | Ayar |
|---|---|---|
| 1 | **Klasörden Dosya Al** *(Get File from Folder)* | Klasör: **Shortcuts** · Yol: `ClipBridge.txt` · **Bulunamazsa Hata Ver**: açık |
| 2 | **Sözlük Değerini Al** *(Get Dictionary Value)* | Girdi: **Dosya** · Anahtar: `url` (JSON dosyası kendiliğinden sözlüğe çevrilir) |
| 3 | **Değişkeni Ayarla** *(Set Variable)* | Ad: `Address` |
| 4 | **Sözlük Değerini Al** | Anahtar: `token`. Girdisi kendiliğinden yanlış dolar: girdiye basılı tut → **Değişken Seç** → 1. adımdaki **Dosya**. |
| 5 | **Değişkeni Ayarla** | Ad: `Token` |

Dosya bulunamazsa kestirme hata verir. Bu, "ClipBridge Setup" kestirmesinin henüz çalıştırılmadığı anlamına gelir.

İpucu: Bir değişkeni bir alana koymak için alana **basılı tut** → **Değişken Seç**. Kısa dokunuş çoğu zaman dosya seçiciyi ya da klavyeyi açar.

## 3. Send to PC

Linkler ayrı bir koldan gider: iOS bir linki Form'un **Dosya** alanına koyamıyor ("WFURLContentItem produced no file representation" hatası), bu yüzden link **Metin** alanıyla gönderilir.

| # | Eylem | Ayar |
|---|---|---|
| 0 | **ⓘ** → **Paylaşma Sayfasında Göster** | Açık. Girdi türleri: hepsi |
| 0 | "Giriş yoksa" | **Panoyu Al** *(Get Clipboard)* |
| 1–5 | Ayarları okuma (yukarıda) | |
| 6 | **Her Biriyle Yinele** *(Repeat with Each)* | Öğe: **Kestirme Girişi** *(Shortcut Input)* |
| 7 | **Metin** *(Text)* | İçerik: yalnızca **Yineleme Öğesi** *(Repeat Item)* |
| 8 | **Eğer** *(If)* | **Metin** **ile başlıyor** *(begins with)* `http` |
| 9 | **URL'nin İçeriğini Al** *(Get Contents of URL)* | URL: `[Address]/clip` · **POST** · Başlık `X-Token` = `[Token]` · **Form** → tür **Metin**, anahtar `file`, değer **Yineleme Öğesi** |
| 10 | **Aksi Hâlde** *(Otherwise)* | |
| 11 | **URL'nin İçeriğini Al** | 9 ile aynı, ama form alanının türü **Dosya** |
| 12 | **Son Eğer**, **Yinelemeyi Bitir** | |

```
Klasörden Dosya Al  Shortcuts/ClipBridge.txt
Dosya içindeki "url" için Değer'i al    →  Değişkeni Ayarla Address
Dosya içindeki "token" için Değer'i al  →  Değişkeni Ayarla Token
Kestirme Girişi girişinde her öğeyle yinele
    Metin: [Yineleme Öğesi]
    Metin ile başlıyor http ise
        [Address]/clip içeriğini al   POST · X-Token = [Token] · Form: file (Metin) = Yineleme Öğesi
    Aksi Hâlde
        [Address]/clip içeriğini al   POST · X-Token = [Token] · Form: file (Dosya) = Yineleme Öğesi
    Son Eğer
Yinelemeyi Bitir
```

Form'da dikkat: `file` **anahtar** alanına, **Yineleme Öğesi** ise **değer** alanına girer. İkisi yer değiştirirse PC "Formda dosya alanı yok" der. Aynısı başlık için de geçerli: anahtar `X-Token`, değer **Token**.

## 4. Get from PC

| # | Eylem | Ayar |
|---|---|---|
| 1–5 | Ayarları okuma (yukarıda) | |
| 6 | **URL'nin İçeriğini Al** | URL: `[Address]/clip` · Yöntem: **GET** · Başlık: `X-Token` = `[Token]` |
| 7 | **Panoya Kopyala** *(Copy to Clipboard)* | Girdi: **URL'nin İçeriği** |
| 8 | **Bildirim Göster** (isteğe bağlı) | `Copied from PC` |

## 5. Tetikleyiciler

Ayarlar → Erişilebilirlik → Dokunma → **Arkaya Dokun**:
- **Çift Dokunma** → Send to PC
- **Üçlü Dokunma** → Get from PC

iPhone 15 Pro ve sonrasında **Eylem Düğmesi** de kullanılabilir.

## 6. Kestirmeleri paylaşmak

Kestirmeler uygulamasında kestirmeye uzun bas → **Paylaş** → **iCloud Bağlantısını Kopyala**. Kestirmelerde gizli bilgi olmadığı için bağlantı herkese verilebilir.

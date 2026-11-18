//! مترجم ومحاكي لوحات راسبيري باي والحواسيب أحادية اللوحة (Raspberry Pi / Linux SBC) للغة لسان.
use crate::ast::{ClassDef, Expr, Function, Op, Stmt};
use std::path::{Path, PathBuf};

/// المكتبة القياسية المدمجة للوحات راسبيري باي (Pi 3 / Pi 4 / Pi 5 / Pi Zero 2 W / Compute Module)
pub const STDLIB_RASPBERRY: &str = r##"
# مكتبة راسبيري باي القياسية في لغة لسان (Raspberry Pi SBC & Linux Hardware)

دع عالي = 1
دع منخفض = 0
دع مخرج_راسبيري = "مخرج"
دع مدخل_راسبيري = "مدخل"
دع سحب_علوي = "علوي"
دع سحب_سفلي = "سفلي"
دع بدون_سحب = "بدون"

دالة انتظر_ثواني(ثواني) {
    انتظر_ملي(ثواني * 1000)
    أرجع صحيح
}

صنف لوحة_راسبيري {
    دالة بناء(طراز = "Raspberry Pi 5") {
        هذا.طراز = طراز
        هذا.حرارة = 44.5
        هذا.استهلاك_ذاكرة = 640
        هذا.أنوية = 4
        إذا (طراز.تحتوي("Zero") && !طراز.تحتوي("Zero 2")) {
            هذا.أنوية = 1
            هذا.استهلاك_ذاكرة = 180
        }
        هذا.عنوان = "192.168.1.150"
        هذا.تعمل = صحيح
    }

    دالة اسم_الطراز() {
        أرجع هذا.طراز
    }

    دالة حرارة_المعالج() {
        أرجع هذا.حرارة
    }

    دالة ضبط_حرارة_محاكاة(حرارة_جديدة) {
        هذا.حرارة = حرارة_جديدة
        أرجع هذا.حرارة
    }

    دالة استهلاك_الذاكرة_ميجا() {
        أرجع هذا.استهلاك_ذاكرة
    }

    دالة عدد_الأنوية() {
        أرجع هذا.أنوية
    }

    دالة عنوان_ip() {
        أرجع هذا.عنوان
    }

    دالة إعادة_تشغيل_اللوحة() {
        هذا.تعمل = صحيح
        أرجع صحيح
    }

    دالة اعادة_تشغيل_اللوحة() {
        أرجع هذا.إعادة_تشغيل_اللوحة()
    }

    دالة إيقاف_التشغيل() {
        هذا.تعمل = خطأ
        أرجع صحيح
    }

    دالة ايقاف_التشغيل() {
        أرجع هذا.إيقاف_التشغيل()
    }
}

صنف منفذ_gpio_راسبيري {
    دالة بناء(رقم_المنفذ، الوضع = "مخرج"، السحب = "بدون") {
        هذا.منفذ = رقم_المنفذ
        هذا.الوضع = الوضع
        هذا.السحب = السحب
        هذا.القيمة = 0
        إذا (الوضع == "مدخل" && السحب == "علوي") {
            هذا.القيمة = 1
        }
        هذا.مهيأ = صحيح
    }

    دالة اكتب(قيمة) {
        دع وضع_القيمة = 0
        إذا (قيمة == 1 || قيمة == صحيح || قيمة == "عالي" || قيمة == "HIGH") {
            وضع_القيمة = 1
        }
        هذا.القيمة = وضع_القيمة
        أرجع هذا.القيمة
    }

    دالة شغّل() {
        أرجع هذا.اكتب(1)
    }

    دالة شغل() {
        أرجع هذا.شغّل()
    }

    دالة أوقف() {
        أرجع هذا.اكتب(0)
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }

    دالة اقلب() {
        إذا (هذا.القيمة == 0) {
            هذا.القيمة = 1
        } وإلا {
            هذا.القيمة = 0
        }
        أرجع هذا.القيمة
    }

    دالة اقرأ() {
        أرجع هذا.القيمة
    }

    دالة اقرا() {
        أرجع هذا.اقرأ()
    }

    دالة ضبط_محاكاة(قيمة) {
        دع وضع_القيمة = 0
        إذا (قيمة == 1 || قيمة == صحيح) {
            وضع_القيمة = 1
        }
        هذا.القيمة = وضع_القيمة
        أرجع هذا.القيمة
    }

    دالة تنظيف() {
        هذا.القيمة = 0
        هذا.مهيأ = خطأ
        أرجع صحيح
    }
}

صنف نبضة_pwm_راسبيري {
    دالة بناء(رقم_المنفذ، تردد = 1000) {
        هذا.منفذ = رقم_المنفذ
        هذا.تردد = تردد
        هذا.نسبة = 0
        هذا.يعمل = خطأ
    }

    دالة ابدأ(نسبة_مئوية = 50) {
        دع ن = نسبة_مئوية
        إذا (ن < 0) { ن = 0 }
        إذا (ن > 100) { ن = 100 }
        هذا.نسبة = ن
        هذا.يعمل = صحيح
        أرجع هذا.نسبة
    }

    دالة ابدا(نسبة_مئوية = 50) {
        أرجع هذا.ابدأ(نسبة_مئوية)
    }

    دالة غير_النسبة(نسبة) {
        دع ن = نسبة
        إذا (ن < 0) { ن = 0 }
        إذا (ن > 100) { ن = 100 }
        هذا.نسبة = ن
        أرجع هذا.نسبة
    }

    دالة غيّر_النسبة(نسبة) {
        أرجع هذا.غير_النسبة(نسبة)
    }

    دالة غير_التردد(تردد_جديد) {
        إذا (تردد_جديد > 0) {
            هذا.تردد = تردد_جديد
        }
        أرجع هذا.تردد
    }

    دالة غيّر_التردد(تردد_جديد) {
        أرجع هذا.غير_التردد(تردد_جديد)
    }

    دالة أوقف() {
        هذا.نسبة = 0
        هذا.يعمل = خطأ
        أرجع صحيح
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }
}

صنف ناقل_i2c_راسبيري {
    دالة بناء(رقم_الناقل = 1) {
        هذا.رقم_الناقل = رقم_الناقل
        هذا.سجلات = {}
        هذا.أجهزة = [60, 64, 118]
    }

    دالة اكتب_بايت(عنوان_الجهاز، سجل، قيمة) {
        دع مفتاح = نص(عنوان_الجهاز) + ":" + نص(سجل)
        دع خريطة = هذا.سجلات
        خريطة[مفتاح] = قيمة
        إذا (!(عنوان_الجهاز في هذا.أجهزة)) {
            هذا.أجهزة.أضف(عنوان_الجهاز)
        }
        أرجع صحيح
    }

    دالة اقرأ_بايت(عنوان_الجهاز، سجل) {
        دع مفتاح = نص(عنوان_الجهاز) + ":" + نص(سجل)
        إذا (مفتاح في هذا.سجلات) {
            أرجع هذا.سجلات[مفتاح]
        }
        أرجع 0
    }

    دالة اقرا_بايت(عنوان_الجهاز، سجل) {
        أرجع هذا.اقرأ_بايت(عنوان_الجهاز، سجل)
    }

    دالة فحص_الأجهزة() {
        أرجع هذا.أجهزة
    }

    دالة فحص_الاجهزة() {
        أرجع هذا.فحص_الأجهزة()
    }
}

صنف ناقل_spi_راسبيري {
    دالة بناء(ناقل = 0، شريحة = 0، سرعة = 1000000) {
        هذا.ناقل = ناقل
        هذا.شريحة = شريحة
        هذا.سرعة = سرعة
        هذا.مفتوح = صحيح
    }

    دالة نقل(قائمة_بايتات) {
        دع رد = []
        لكل بايت في قائمة_بايتات {
            رد.أضف(بايت)
        }
        أرجع رد
    }

    دالة أغلق() {
        هذا.مفتوح = خطأ
        أرجع صحيح
    }

    دالة اغلق() {
        أرجع هذا.أغلق()
    }
}

صنف منفذ_تسلسلي_راسبيري {
    دالة بناء(مسار = "/dev/serial0"، سرعة = 115200) {
        هذا.مسار = مسار
        هذا.سرعة = سرعة
        هذا.مفتوح = صحيح
        هذا.آخر_مرسل = ""
        هذا.آخر_مستقبل = "OK"
    }

    دالة اكتب_سطر(رسالة) {
        هذا.آخر_مرسل = نص(رسالة)
        أرجع صحيح
    }

    دالة اقرأ_سطر() {
        أرجع هذا.آخر_مستقبل
    }

    دالة اقرا_سطر() {
        أرجع هذا.اقرأ_سطر()
    }

    دالة ضبط_محاكاة_استقبال(نص_مستقبل) {
        هذا.آخر_مستقبل = نص(نص_مستقبل)
        أرجع هذا.آخر_مستقبل
    }

    دالة أغلق() {
        هذا.مفتوح = خطأ
        أرجع صحيح
    }

    دالة اغلق() {
        أرجع هذا.أغلق()
    }
}

صنف كاميرا_راسبيري {
    دالة بناء(عرض = 1920، ارتفاع = 1080) {
        هذا.عرض = عرض
        هذا.ارتفاع = ارتفاع
        هذا.تعمل = خطأ
        هذا.تبث = خطأ
        هذا.منفذ_البث = 8000
        هذا.آخر_صورة = ""
        هذا.آخر_فيديو = ""
        هذا.عدد_اللقطات = 0
    }

    دالة ابدأ() {
        هذا.تعمل = صحيح
        أرجع صحيح
    }

    دالة ابدا() {
        أرجع هذا.ابدأ()
    }

    دالة التقط_صورة(مسار_الملف) {
        هذا.تعمل = صحيح
        هذا.آخر_صورة = مسار_الملف
        هذا.عدد_اللقطات = هذا.عدد_اللقطات + 1
        أرجع صحيح
    }

    دالة سجل_فيديو(مسار_الملف، مدة_ثواني = 5) {
        هذا.تعمل = صحيح
        هذا.آخر_فيديو = مسار_الملف
        هذا.آخر_مدة = مدة_ثواني
        أرجع صحيح
    }

    دالة سجّل_فيديو(مسار_الملف، مدة_ثواني = 5) {
        أرجع هذا.سجل_فيديو(مسار_الملف، مدة_ثواني)
    }

    دالة ابدأ_بث(منفذ = 8000) {
        هذا.تعمل = صحيح
        هذا.تبث = صحيح
        هذا.منفذ_البث = منفذ
        أرجع صحيح
    }

    دالة ابدا_بث(منفذ = 8000) {
        أرجع هذا.ابدأ_بث(منفذ)
    }

    دالة أوقف() {
        هذا.تبث = خطأ
        هذا.تعمل = خطأ
        أرجع صحيح
    }

    دالة اوقف() {
        أرجع هذا.أوقف()
    }
}

صنف قبعة_الحساسات_sensehat {
    دالة بناء() {
        هذا.حرارة = 25.4
        هذا.رطوبة = 48.2
        هذا.ضغط = 1013.25
        هذا.انحدار = 1.5
        هذا.ميلان = 0.8
        هذا.انعطاف = 180.0
        هذا.آخر_رسالة = ""
        هذا.مصفوفة_نشطة = خطأ
    }

    دالة اعرض_رسالة(نص_رسالة، سرعة = 0.1) {
        هذا.آخر_رسالة = نص(نص_رسالة)
        هذا.سرعة_العرض = سرعة
        هذا.مصفوفة_نشطة = صحيح
        أرجع صحيح
    }

    دالة اضبط_بكسل(س، ص، أحمر، أخضر، أزرق) {
        هذا.مصفوفة_نشطة = صحيح
        هذا.آخر_بكسل = [س، ص، أحمر، أخضر، أزرق]
        أرجع صحيح
    }

    دالة امسح_الشاشة() {
        هذا.مصفوفة_نشطة = خطأ
        أرجع صحيح
    }

    دالة اقرأ_حرارة() {
        أرجع هذا.حرارة
    }

    دالة اقرأ_رطوبة() {
        أرجع هذا.رطوبة
    }

    دالة اقرأ_ضغط() {
        أرجع هذا.ضغط
    }

    دالة اقرأ_توجيه() {
        أرجع {"انحدار": هذا.انحدار، "ميلان": هذا.ميلان، "انعطاف": هذا.انعطاف}
    }

    دالة ضبط_محاكاة_بيئة(حرارة، رطوبة، ضغط) {
        هذا.حرارة = حرارة
        هذا.رطوبة = رطوبة
        هذا.ضغط = ضغط
        أرجع صحيح
    }
}

صنف قبعة_محركات_pca9685 {
    دالة بناء(عنوان = 64) {
        هذا.عنوان = عنوان
        هذا.زوايا = {}
        هذا.سرعات = {}
    }

    دالة اضبط_زاوية_سيرفو(قناة، زاوية) {
        دع ز = زاوية
        إذا (ز < 0) { ز = 0 }
        إذا (ز > 180) { ز = 180 }
        دع مفتاح = نص(قناة)
        دع خ = هذا.زوايا
        خ[مفتاح] = ز
        أرجع ز
    }

    دالة اضبط_سرعة_محرك(قناة، نسبة) {
        دع ن = نسبة
        إذا (ن < 0) { ن = 0 }
        إذا (ن > 100) { ن = 100 }
        دع مفتاح = نص(قناة)
        دع خ = هذا.سرعات
        خ[مفتاح] = ن
        أرجع ن
    }
}

صنف شاشة_لمس_dsi {
    دالة بناء(حجم_بوصة = 7) {
        هذا.حجم_بوصة = حجم_بوصة
        هذا.سطوع = 80
        هذا.مشتغلة = صحيح
    }

    دالة اضبط_سطوع(نسبة) {
        دع ن = نسبة
        إذا (ن < 0) { ن = 0 }
        إذا (ن > 100) { ن = 100 }
        هذا.سطوع = ن
        أرجع هذا.سطوع
    }

    دالة شغّل_الشاشة(وضع_التشغيل) {
        دع مفعلة = خطأ
        إذا (وضع_التشغيل == صحيح || وضع_التشغيل == 1) {
            مفعلة = صحيح
        }
        هذا.مشتغلة = مفعلة
        أرجع هذا.مشتغلة
    }

    دالة شغل_الشاشة(وضع_التشغيل) {
        أرجع هذا.شغّل_الشاشة(وضع_التشغيل)
    }
}

صنف صوت_راسبيري {
    دالة بناء(مستوى_الصوت = 80) {
        دع م = مستوى_الصوت
        إذا (م < 0) { م = 0 }
        إذا (م > 100) { م = 100 }
        هذا.مستوى_الصوت = م
        هذا.آخر_ملف = ""
        هذا.آخر_نطق = ""
    }

    دالة اضبط_الصوت(نسبة) {
        دع م = نسبة
        إذا (م < 0) { م = 0 }
        إذا (م > 100) { م = 100 }
        هذا.مستوى_الصوت = م
        أرجع هذا.مستوى_الصوت
    }

    دالة شغّل_ملف(مسار) {
        هذا.آخر_ملف = مسار
        أرجع صحيح
    }

    دالة شغل_ملف(مسار) {
        أرجع هذا.شغّل_ملف(مسار)
    }

    دالة انطق_عربي(نص_رسالة) {
        هذا.آخر_نطق = نص(نص_رسالة)
        أرجع صحيح
    }
}

صنف شاشة_حبر_إلكتروني_epaper {
    دالة بناء(عرض = 250، ارتفاع = 122) {
        هذا.عرض = عرض
        هذا.ارتفاع = ارتفاع
        هذا.نصوص = []
        هذا.في_وضع_السكون = خطأ
        هذا.عدد_التحديثات = 0
    }

    دالة امسح() {
        هذا.نصوص.مسح()
        هذا.في_وضع_السكون = خطأ
        أرجع صحيح
    }

    دالة اعرض_نص(س، ص، نص_رسالة) {
        هذا.في_وضع_السكون = خطأ
        هذا.نصوص.أضف({"س": س، "ص": ص، "نص": نص(نص_رسالة)})
        أرجع صحيح
    }

    دالة تحديث_الشاشة() {
        هذا.عدد_التحديثات = هذا.عدد_التحديثات + 1
        أرجع صحيح
    }

    دالة سكون() {
        هذا.في_وضع_السكون = صحيح
        أرجع صحيح
    }
}
"##;

/// الرأس البرمجي المولّد في بايثون 3 لدعم عتاد راسبيري باي الحقيقي مع التراجع الآمن للمحاكاة
const PYTHON_RPI_RUNTIME: &str = r#"# ==============================================================================
# تم توليد هذا السكربت تلقائياً بواسطة مترجم راسبيري باي في لغة لسان (Lisan)
# يدعم التشغيل المباشر على Raspberry Pi OS (Pi 3 / 4 / 5 / Zero 2 W / CM)
# مع وضع المحاكاة التلقائي الآمن على أنظمة سطح المكتب (Linux / macOS / Windows)
# ==============================================================================

import math
import os
import socket
import subprocess
import sys
import time
from pathlib import Path

# الكشف الذكي عن العمل فوق لوحة راسبيري باي حقيقية
_IS_REAL_RPI = False
try:
    _dt_model = Path("/sys/firmware/devicetree/base/model")
    if _dt_model.exists():
        _model_txt = _dt_model.read_text(encoding="utf-8", errors="ignore")
        if "Raspberry Pi" in _model_txt:
            _IS_REAL_RPI = True
except Exception:
    pass
if os.environ.get("LISAN_FORCE_RPI_HW") == "1":
    _IS_REAL_RPI = True

# استيراد مكتبات عتاد راسبيري باي مع التراجع الآمن عند عدم توفرها
try:
    import gpiozero as _gpiozero
    _HAS_GPIOZERO = True
except Exception:
    _gpiozero = None
    _HAS_GPIOZERO = False

try:
    import RPi.GPIO as _RPI_GPIO
    if _IS_REAL_RPI:
        _RPI_GPIO.setwarnings(False)
        _RPI_GPIO.setmode(_RPI_GPIO.BCM)
    _HAS_RPI_GPIO = True
except Exception:
    _RPI_GPIO = None
    _HAS_RPI_GPIO = False

try:
    import smbus2 as _smbus2
    _HAS_SMBUS = True
except Exception:
    _smbus2 = None
    _HAS_SMBUS = False

try:
    import spidev as _spidev
    _HAS_SPIDEV = True
except Exception:
    _spidev = None
    _HAS_SPIDEV = False

try:
    import serial as _pyserial
    _HAS_SERIAL = True
except Exception:
    _pyserial = None
    _HAS_SERIAL = False

try:
    from picamera2 import Picamera2 as _Picamera2
    _HAS_PICAMERA2 = True
except Exception:
    _Picamera2 = None
    _HAS_PICAMERA2 = False

try:
    from sense_hat import SenseHat as _SenseHat
    _HAS_SENSE_HAT = True
except Exception:
    _SenseHat = None
    _HAS_SENSE_HAT = False

عالي = 1
منخفض = 0
مخرج_راسبيري = "مخرج"
مدخل_راسبيري = "مدخل"
سحب_علوي = "علوي"
سحب_سفلي = "سفلي"
بدون_سحب = "بدون"


def _lsn_add(a, b):
    if isinstance(a, str) or isinstance(b, str):
        return f"{a}{b}"
    return a + b


def انتظر_ثواني(ثواني):
    time.sleep(max(0.0, float(ثواني)))
    return True


class لوحة_راسبيري:
    def __init__(self, طراز="Raspberry Pi 5"):
        self.طراز = طراز
        if _IS_REAL_RPI:
            try:
                raw = Path("/sys/firmware/devicetree/base/model").read_text(encoding="utf-8", errors="ignore").strip("\x00\r\n ")
                if raw:
                    self.طراز = raw
            except Exception:
                pass
        self._sim_temp = 44.5
        self.تعمل = True

    def اسم_الطراز(self):
        return self.طراز

    def حرارة_المعالج(self):
        thermal = Path("/sys/class/thermal/thermal_zone0/temp")
        if _IS_REAL_RPI and thermal.exists():
            try:
                return round(float(thermal.read_text().strip()) / 1000.0, 1)
            except Exception:
                pass
        return self._sim_temp

    def ضبط_حرارة_محاكاة(self, حرارة_جديدة):
        self._sim_temp = float(حرارة_جديدة)
        return self._sim_temp

    def استهلاك_الذاكرة_ميجا(self):
        meminfo = Path("/proc/meminfo")
        if _IS_REAL_RPI and meminfo.exists():
            try:
                info = {}
                for line in meminfo.read_text().splitlines():
                    parts = line.split(":")
                    if len(parts) == 2:
                        info[parts[0].strip()] = int(parts[1].strip().split()[0])
                if "MemTotal" in info and "MemAvailable" in info:
                    return round((info["MemTotal"] - info["MemAvailable"]) / 1024.0, 1)
            except Exception:
                pass
        return 640.0

    def عدد_الأنوية(self):
        return os.cpu_count() or 4

    def عنوان_ip(self):
        if _IS_REAL_RPI:
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                sock.settimeout(0.2)
                sock.connect(("8.8.8.8", 80))
                ip = sock.getsockname()[0]
                sock.close()
                return ip
            except Exception:
                pass
        return "192.168.1.150"

    def إعادة_تشغيل_اللوحة(self):
        if _IS_REAL_RPI and os.environ.get("LISAN_RPI_ALLOW_POWER") == "1":
            subprocess.run(["sudo", "reboot"], check=False)
        return True

    اعادة_تشغيل_اللوحة = إعادة_تشغيل_اللوحة

    def إيقاف_التشغيل(self):
        self.تعمل = False
        if _IS_REAL_RPI and os.environ.get("LISAN_RPI_ALLOW_POWER") == "1":
            subprocess.run(["sudo", "shutdown", "-h", "now"], check=False)
        return True

    ايقاف_التشغيل = إيقاف_التشغيل


class منفذ_gpio_راسبيري:
    def __init__(self, رقم_المنفذ, الوضع="مخرج", السحب="بدون"):
        self.منفذ = int(رقم_المنفذ)
        self.الوضع = الوضع
        self.السحب = السحب
        self.القيمة = 1 if (الوضع == "مدخل" and السحب == "علوي") else 0
        self._dev = None
        self._use_rpi_gpio = False
        if _IS_REAL_RPI:
            if _HAS_GPIOZERO:
                try:
                    if الوضع in ("مخرج", "OUTPUT"):
                        self._dev = _gpiozero.DigitalOutputDevice(self.منفذ)
                    else:
                        pull_up = True if السحب == "علوي" else (False if السحب == "سفلي" else None)
                        self._dev = _gpiozero.DigitalInputDevice(self.منفذ, pull_up=pull_up)
                except Exception:
                    self._dev = None
            elif _HAS_RPI_GPIO:
                try:
                    if الوضع in ("مخرج", "OUTPUT"):
                        _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.OUT, initial=_RPI_GPIO.LOW)
                    else:
                        pud = _RPI_GPIO.PUD_OFF
                        if السحب == "علوي":
                            pud = _RPI_GPIO.PUD_UP
                        elif السحب == "سفلي":
                            pud = _RPI_GPIO.PUD_DOWN
                        _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.IN, pull_up_down=pud)
                    self._use_rpi_gpio = True
                except Exception:
                    self._use_rpi_gpio = False

    def اكتب(self, قيمة):
        state = 1 if قيمة in (1, True, "عالي", "HIGH") else 0
        self.القيمة = state
        if self._dev is not None:
            try:
                self._dev.on() if state else self._dev.off()
            except Exception:
                pass
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                _RPI_GPIO.output(self.منفذ, _RPI_GPIO.HIGH if state else _RPI_GPIO.LOW)
            except Exception:
                pass
        return self.القيمة

    def شغّل(self):
        return self.اكتب(1)

    شغل = شغّل

    def أوقف(self):
        return self.اكتب(0)

    اوقف = أوقف

    def اقلب(self):
        return self.اكتب(0 if self.القيمة else 1)

    def اقرأ(self):
        if self._dev is not None:
            try:
                self.القيمة = 1 if self._dev.value else 0
            except Exception:
                pass
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                self.القيمة = 1 if _RPI_GPIO.input(self.منفذ) else 0
            except Exception:
                pass
        return self.القيمة

    اقرا = اقرأ

    def ضبط_محاكاة(self, قيمة):
        self.القيمة = 1 if قيمة in (1, True) else 0
        return self.القيمة

    def تنظيف(self):
        self.القيمة = 0
        if self._dev is not None:
            try:
                self._dev.close()
            except Exception:
                pass
            self._dev = None
        elif self._use_rpi_gpio and _HAS_RPI_GPIO:
            try:
                _RPI_GPIO.cleanup(self.منفذ)
            except Exception:
                pass
            self._use_rpi_gpio = False
        return True


class نبضة_pwm_راسبيري:
    def __init__(self, رقم_المنفذ, تردد=1000):
        self.منفذ = int(رقم_المنفذ)
        self.تردد = max(1, int(تردد))
        self.نسبة = 0.0
        self.يعمل = False
        self._pwm = None
        self._is_gpiozero = False
        if _IS_REAL_RPI:
            if _HAS_GPIOZERO:
                try:
                    self._pwm = _gpiozero.PWMOutputDevice(self.منفذ, frequency=self.تردد)
                    self._is_gpiozero = True
                except Exception:
                    self._pwm = None
            elif _HAS_RPI_GPIO:
                try:
                    _RPI_GPIO.setup(self.منفذ, _RPI_GPIO.OUT)
                    self._pwm = _RPI_GPIO.PWM(self.منفذ, self.تردد)
                except Exception:
                    self._pwm = None

    def ابدأ(self, نسبة_مئوية=50):
        self.نسبة = max(0.0, min(100.0, float(نسبة_مئوية)))
        self.يعمل = True
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.value = self.نسبة / 100.0
                else:
                    self._pwm.start(self.نسبة)
            except Exception:
                pass
        return self.نسبة

    ابدا = ابدأ

    def غير_النسبة(self, نسبة):
        self.نسبة = max(0.0, min(100.0, float(نسبة)))
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.value = self.نسبة / 100.0
                else:
                    self._pwm.ChangeDutyCycle(self.نسبة)
            except Exception:
                pass
        return self.نسبة

    غيّر_النسبة = غير_النسبة

    def غير_التردد(self, تردد_جديد):
        if تردد_جديد > 0:
            self.تردد = int(تردد_جديد)
            if self._pwm is not None:
                try:
                    if self._is_gpiozero:
                        self._pwm.frequency = self.تردد
                    else:
                        self._pwm.ChangeFrequency(self.تردد)
                except Exception:
                    pass
        return self.تردد

    غيّر_التردد = غير_التردد

    def أوقف(self):
        self.نسبة = 0.0
        self.يعمل = False
        if self._pwm is not None:
            try:
                if self._is_gpiozero:
                    self._pwm.off()
                else:
                    self._pwm.stop()
            except Exception:
                pass
        return True

    اوقف = أوقف


class ناقل_i2c_راسبيري:
    def __init__(self, رقم_الناقل=1):
        self.رقم_الناقل = int(رقم_الناقل)
        self._regs = {}
        self._devices = [0x3C, 0x40, 0x76]
        self._bus = None
        if _IS_REAL_RPI and _HAS_SMBUS and Path(f"/dev/i2c-{self.رقم_الناقل}").exists():
            try:
                self._bus = _smbus2.SMBus(self.رقم_الناقل)
            except Exception:
                self._bus = None

    def اكتب_بايت(self, عنوان_الجهاز, سجل, قيمة):
        addr = int(عنوان_الجهاز)
        reg = int(سجل)
        val = int(قيمة) & 0xFF
        self._regs[(addr, reg)] = val
        if addr not in self._devices:
            self._devices.append(addr)
        if self._bus is not None:
            try:
                self._bus.write_byte_data(addr, reg, val)
            except Exception:
                pass
        return True

    def اقرأ_بايت(self, عنوان_الجهاز, سجل):
        addr = int(عنوان_الجهاز)
        reg = int(سجل)
        if self._bus is not None:
            try:
                return self._bus.read_byte_data(addr, reg)
            except Exception:
                pass
        return self._regs.get((addr, reg), 0)

    اقرا_بايت = اقرأ_بايت

    def فحص_الأجهزة(self):
        if self._bus is not None:
            found = []
            for addr in range(0x03, 0x78):
                try:
                    self._bus.read_byte(addr)
                    found.append(addr)
                except Exception:
                    pass
            if found:
                return found
        return list(self._devices)

    فحص_الاجهزة = فحص_الأجهزة


class ناقل_spi_راسبيري:
    def __init__(self, ناقل=0, شريحة=0, سرعة=1000000):
        self.ناقل = int(ناقل)
        self.شريحة = int(شريحة)
        self.سرعة = int(سرعة)
        self.مفتوح = True
        self._spi = None
        if _IS_REAL_RPI and _HAS_SPIDEV and Path(f"/dev/spidev{self.ناقل}.{self.شريحة}").exists():
            try:
                self._spi = _spidev.SpiDev()
                self._spi.open(self.ناقل, self.شريحة)
                self._spi.max_speed_hz = self.سرعة
            except Exception:
                self._spi = None

    def نقل(self, قائمة_بايتات):
        payload = [int(b) & 0xFF for b in قائمة_بايتات]
        if self._spi is not None and self.مفتوح:
            try:
                return list(self._spi.xfer2(payload))
            except Exception:
                pass
        return list(payload)

    def أغلق(self):
        self.مفتوح = False
        if self._spi is not None:
            try:
                self._spi.close()
            except Exception:
                pass
            self._spi = None
        return True

    اغلق = أغلق


class منفذ_تسلسلي_راسبيري:
    def __init__(self, مسار="/dev/serial0", سرعة=115200):
        self.مسار = str(مسار)
        self.سرعة = int(سرعة)
        self.مفتوح = True
        self.آخر_مرسل = ""
        self.آخر_مستقبل = "OK"
        self._ser = None
        if _IS_REAL_RPI and _HAS_SERIAL and Path(self.مسار).exists():
            try:
                self._ser = _pyserial.Serial(self.مسار, self.سرعة, timeout=1.0)
            except Exception:
                self._ser = None

    def اكتب_سطر(self, رسالة):
        line = f"{رسالة}\n"
        self.آخر_مرسل = str(رسالة)
        if self._ser is not None and self.مفتوح:
            try:
                self._ser.write(line.encode("utf-8"))
            except Exception:
                pass
        return True

    def اقرأ_سطر(self):
        if self._ser is not None and self.مفتوح:
            try:
                raw = self._ser.readline()
                if raw:
                    return raw.decode("utf-8", errors="replace").rstrip("\r\n")
            except Exception:
                pass
        return self.آخر_مستقبل

    اقرا_سطر = اقرأ_سطر

    def ضبط_محاكاة_استقبال(self, نص_مستقبل):
        self.آخر_مستقبل = str(نص_مستقبل)
        return self.آخر_مستقبل

    def أغلق(self):
        self.مفتوح = False
        if self._ser is not None:
            try:
                self._ser.close()
            except Exception:
                pass
            self._ser = None
        return True

    اغلق = أغلق


class كاميرا_راسبيري:
    def __init__(self, عرض=1920, ارتفاع=1080):
        self.عرض = int(عرض)
        self.ارتفاع = int(ارتفاع)
        self.تعمل = False
        self.تبث = False
        self.منفذ_البث = 8000
        self.آخر_صورة = ""
        self.آخر_فيديو = ""
        self._cam = None
        if _IS_REAL_RPI and _HAS_PICAMERA2:
            try:
                self._cam = _Picamera2()
                cfg = self._cam.create_still_configuration(main={"size": (self.عرض, self.ارتفاع)})
                self._cam.configure(cfg)
            except Exception:
                self._cam = None

    def ابدأ(self):
        self.تعمل = True
        if self._cam is not None:
            try:
                self._cam.start()
            except Exception:
                pass
        return True

    ابدا = ابدأ

    def التقط_صورة(self, مسار_الملف):
        self.تعمل = True
        self.آخر_صورة = str(مسار_الملف)
        if self._cam is not None:
            try:
                self._cam.capture_file(self.آخر_صورة)
            except Exception:
                pass
        return True

    def سجل_فيديو(self, مسار_الملف, مدة_ثواني=5):
        self.تعمل = True
        self.آخر_فيديو = str(مسار_الملف)
        if self._cam is not None:
            try:
                self._cam.start_and_record_video(self.آخر_فيديو, duration=float(مدة_ثواني))
            except Exception:
                pass
        return True

    سجّل_فيديو = سجل_فيديو

    def ابدأ_بث(self, منفذ=8000):
        self.تعمل = True
        self.تبث = True
        self.منفذ_البث = int(منفذ)
        return True

    ابدا_بث = ابدأ_بث

    def أوقف(self):
        self.تبث = False
        self.تعمل = False
        if self._cam is not None:
            try:
                self._cam.stop()
            except Exception:
                pass
        return True

    اوقف = أوقف


class قبعة_الحساسات_sensehat:
    def __init__(self):
        self.حرارة = 25.4
        self.رطوبة = 48.2
        self.ضغط = 1013.25
        self._hat = None
        if _IS_REAL_RPI and _HAS_SENSE_HAT:
            try:
                self._hat = _SenseHat()
            except Exception:
                self._hat = None

    def اعرض_رسالة(self, نص_رسالة, سرعة=0.1):
        if self._hat is not None:
            try:
                self._hat.show_message(str(نص_رسالة), scroll_speed=float(سرعة))
            except Exception:
                pass
        return True

    def اضبط_بكسل(self, س, ص, أحمر, أخضر, أزرق):
        if self._hat is not None:
            try:
                self._hat.set_pixel(int(س) % 8, int(ص) % 8, int(أحمر) & 255, int(أخضر) & 255, int(أزرق) & 255)
            except Exception:
                pass
        return True

    def امسح_الشاشة(self):
        if self._hat is not None:
            try:
                self._hat.clear()
            except Exception:
                pass
        return True

    def اقرأ_حرارة(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_temperature()), 2)
            except Exception:
                pass
        return self.حرارة

    def اقرأ_رطوبة(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_humidity()), 2)
            except Exception:
                pass
        return self.رطوبة

    def اقرأ_ضغط(self):
        if self._hat is not None:
            try:
                return round(float(self._hat.get_pressure()), 2)
            except Exception:
                pass
        return self.ضغط

    def اقرأ_توجيه(self):
        if self._hat is not None:
            try:
                o = self._hat.get_orientation()
                return {
                    "انحدار": round(float(o.get("pitch", 0.0)), 2),
                    "ميلان": round(float(o.get("roll", 0.0)), 2),
                    "انعطاف": round(float(o.get("yaw", 0.0)), 2),
                }
            except Exception:
                pass
        return {"انحدار": 1.5, "ميلان": 0.8, "انعطاف": 180.0}

    def ضبط_محاكاة_بيئة(self, حرارة, رطوبة, ضغط):
        self.حرارة = float(حرارة)
        self.رطوبة = float(رطوبة)
        self.ضغط = float(ضغط)
        return True


class قبعة_محركات_pca9685:
    def __init__(self, عنوان=64):
        self.عنوان = int(عنوان)
        self.زوايا = {}
        self.سرعات = {}
        self._bus = None
        if _IS_REAL_RPI and _HAS_SMBUS and Path("/dev/i2c-1").exists():
            try:
                self._bus = _smbus2.SMBus(1)
                self._bus.write_byte_data(self.عنوان, 0x00, 0x10)
                self._bus.write_byte_data(self.عنوان, 0xFE, 121)  # 50Hz للسيرفو
                self._bus.write_byte_data(self.عنوان, 0x00, 0x20)
            except Exception:
                self._bus = None

    def _set_pwm(self, channel, on_tick, off_tick):
        if self._bus is not None:
            base = 0x06 + 4 * (int(channel) & 0x0F)
            try:
                self._bus.write_byte_data(self.عنوان, base, on_tick & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 1, (on_tick >> 8) & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 2, off_tick & 0xFF)
                self._bus.write_byte_data(self.عنوان, base + 3, (off_tick >> 8) & 0xFF)
            except Exception:
                pass

    def اضبط_زاوية_سيرفو(self, قناة, زاوية):
        ang = max(0.0, min(180.0, float(زاوية)))
        self.زوايا[int(قناة)] = ang
        pulse = int(150 + (ang / 180.0) * 450)
        self._set_pwm(قناة, 0, pulse)
        return ang

    def اضبط_سرعة_محرك(self, قناة, نسبة):
        pct = max(0.0, min(100.0, float(نسبة)))
        self.سرعات[int(قناة)] = pct
        duty = int((pct / 100.0) * 4095)
        self._set_pwm(قناة, 0, duty)
        return pct


class شاشة_لمس_dsi:
    def __init__(self, حجم_بوصة=7):
        self.حجم_بوصة = حجم_بوصة
        self.سطوع = 80
        self.مشتغلة = True

    def اضبط_سطوع(self, نسبة):
        pct = max(0, min(100, int(نسبة)))
        self.سطوع = pct
        if _IS_REAL_RPI:
            raw_val = int((pct / 100.0) * 255)
            for bl in (
                Path("/sys/class/backlight/rpi_backlight/brightness"),
                Path("/sys/class/backlight/10-0045/brightness"),
            ):
                if bl.exists():
                    try:
                        bl.write_text(f"{raw_val}\n")
                        break
                    except Exception:
                        pass
        return self.سطوع

    def شغّل_الشاشة(self, حالة):
        on = bool(حالة in (True, 1))
        self.مشتغلة = on
        if _IS_REAL_RPI:
            bl_power = Path("/sys/class/backlight/rpi_backlight/bl_power")
            if bl_power.exists():
                try:
                    bl_power.write_text("0\n" if on else "1\n")
                except Exception:
                    pass
        return self.مشتغلة

    شغل_الشاشة = شغّل_الشاشة


class صوت_راسبيري:
    def __init__(self, مستوى_الصوت=80):
        self.مستوى_الصوت = max(0, min(100, int(مستوى_الصوت)))
        self.آخر_ملف = ""
        self.آخر_نطق = ""

    def اضبط_الصوت(self, نسبة):
        self.مستوى_الصوت = max(0, min(100, int(نسبة)))
        if _IS_REAL_RPI:
            try:
                subprocess.run(["amixer", "-q", "sset", "Master", f"{self.مستوى_الصوت}%"], check=False)
            except Exception:
                pass
        return self.مستوى_الصوت

    def شغّل_ملف(self, مسار):
        self.آخر_ملف = str(مسار)
        if _IS_REAL_RPI and Path(self.آخر_ملف).exists():
            player = "aplay" if self.آخر_ملف.lower().endswith(".wav") else "mpg123"
            try:
                subprocess.run([player, "-q", self.آخر_ملف], check=False)
            except Exception:
                pass
        return True

    شغل_ملف = شغّل_ملف

    def انطق_عربي(self, نص_رسالة):
        self.آخر_نطق = str(نص_رسالة)
        if _IS_REAL_RPI:
            try:
                subprocess.run(["espeak-ng", "-v", "ar", self.آخر_نطق], check=False)
            except Exception:
                pass
        return True


class شاشة_حبر_إلكتروني_epaper:
    def __init__(self, عرض=250, ارتفاع=122):
        self.عرض = int(عرض)
        self.ارتفاع = int(ارتفاع)
        self.نصوص = []
        self.في_وضع_السكون = False
        self.عدد_التحديثات = 0

    def امسح(self):
        self.نصوص.clear()
        self.في_وضع_السكون = False
        return True

    def اعرض_نص(self, س, ص, نص_رسالة):
        self.في_وضع_السكون = False
        self.نصوص.append({"س": int(س), "ص": int(ص), "نص": str(نص_رسالة)})
        return True

    def تحديث_الشاشة(self):
        self.عدد_التحديثات += 1
        return True

    def سكون(self):
        self.في_وضع_السكون = True
        return True
"#;

/// تحويل شيفرة لسان إلى سكربت بايثون 3 تنفيذي مخصص للوحات راسبيري باي (Raspberry Pi OS)
pub fn transpile_raspberry_python(source: &str) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;
    emit_python_program(&statements)
}

/// بناء وحفظ ملف بايثون راسبيري باي (`.py`) قابل للتنفيذ من ملف لسان (`.lsn`)
pub fn build_raspberry_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let py_code = transpile_raspberry_python(&source)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("raspberry");
            input.with_file_name(format!("{}_rpi.py", stem))
        }
    };

    std::fs::write(&out_path, py_code)
        .map_err(|e| format!("تعذر حفظ ملف راسبيري باي '{}': {}", out_path.display(), e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&out_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = std::fs::set_permissions(&out_path, perms);
        }
    }

    Ok(out_path)
}

fn emit_python_program(statements: &[Stmt]) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("#!/usr/bin/env python3\n");
    out.push_str("# -*- coding: utf-8 -*-\n");
    out.push_str(PYTHON_RPI_RUNTIME);
    out.push_str("\n# --- بداية شيفرة المستخدم المترجمة من لغة لسان ---\n\n");

    for stmt in statements {
        let rendered = emit_py_stmt(stmt, 0, false);
        if !rendered.is_empty() {
            out.push_str(&rendered);
        }
    }

    Ok(out)
}

fn is_raspberry_builtin_module(name: &str) -> bool {
    let trimmed = name.trim().strip_suffix(".lsn").unwrap_or(name.trim());
    let norm = crate::diagnostics::normalize_arabic(trimmed);
    matches!(
        norm.as_str(),
        "راسبيري"
            | "راسبيري_باي"
            | "raspberry"
            | "raspberry_pi"
            | "rpi"
            | "اردوينو"
            | "arduino"
            | "متحكمات"
            | "embedded"
    )
}

fn emit_py_body(stmts: &[Stmt], indent: usize, in_init: bool) -> String {
    let pad = "    ".repeat(indent);
    if stmts.is_empty() {
        return format!("{}pass\n", pad);
    }
    let mut s = String::new();
    for inner in stmts {
        s.push_str(&emit_py_stmt(inner, indent, in_init));
    }
    if s.trim().is_empty() {
        format!("{}pass\n", pad)
    } else {
        s
    }
}

fn emit_py_stmt(stmt: &Stmt, indent: usize, in_init: bool) -> String {
    let pad = "    ".repeat(indent);
    match stmt {
        Stmt::Spanned(inner, _, _) => emit_py_stmt(inner, indent, in_init),
        Stmt::Import(module) => {
            if is_raspberry_builtin_module(module) {
                String::new()
            } else {
                format!("{}# استورد {}\n", pad, module)
            }
        }
        Stmt::ImportModule { module, alias } => {
            if is_raspberry_builtin_module(module) {
                String::new()
            } else if let Some(al) = alias {
                format!("{}import {} as {}\n", pad, py_ident(module), py_ident(al))
            } else {
                format!("{}import {}\n", pad, py_ident(module))
            }
        }
        Stmt::FromImport { module, items } => {
            if is_raspberry_builtin_module(module) {
                String::new()
            } else {
                let py_items: Vec<String> = items.iter().map(|i| py_ident(i)).collect();
                format!(
                    "{}from {} import {}\n",
                    pad,
                    py_ident(module),
                    py_items.join(", ")
                )
            }
        }
        Stmt::Let(name, expr) => {
            format!("{}{} = {}\n", pad, py_ident(name), emit_py_expr(expr))
        }
        Stmt::LetUnpack(names, expr) => {
            let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
            format!("{}{} = {}\n", pad, lhs.join(", "), emit_py_expr(expr))
        }
        Stmt::Print(expr) => {
            format!("{}print({})\n", pad, emit_py_expr(expr))
        }
        Stmt::Expr(expr) => match expr.unwrap_spanned() {
            Expr::Assign(name, val) => {
                format!("{}{} = {}\n", pad, py_ident(name), emit_py_expr(val))
            }
            Expr::SetProperty(target, prop, val) => {
                format!(
                    "{}{}.{} = {}\n",
                    pad,
                    emit_py_expr(target),
                    py_ident(prop),
                    emit_py_expr(val)
                )
            }
            Expr::AssignIndex(name, idx, val) => {
                format!(
                    "{}{}[{}] = {}\n",
                    pad,
                    py_ident(name),
                    emit_py_expr(idx),
                    emit_py_expr(val)
                )
            }
            Expr::AssignUnpack(names, val) => {
                let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
                format!("{}{} = {}\n", pad, lhs.join(", "), emit_py_expr(val))
            }
            other => format!("{}{}\n", pad, emit_py_expr(other)),
        },
        Stmt::Block(stmts) => emit_py_body(stmts, indent, in_init),
        Stmt::If(cond, yes, no) => emit_py_if(cond, yes, no.as_deref(), indent, in_init, "if"),
        Stmt::While(cond, body) => {
            let mut s = format!("{}while {}:\n", pad, emit_py_expr(cond));
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::ForEach(var, iter, body) => {
            let mut s = format!(
                "{}for {} in {}:\n",
                pad,
                py_ident(var),
                emit_py_expr(iter)
            );
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::ForEachUnpack(vars, iter, body) => {
            let py_vars: Vec<String> = vars.iter().map(|v| py_ident(v)).collect();
            let mut s = format!(
                "{}for {} in {}:\n",
                pad,
                py_vars.join(", "),
                emit_py_expr(iter)
            );
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::With { expr, alias, body } => {
            let mut s = if let Some(al) = alias {
                format!(
                    "{}with {} as {}:\n",
                    pad,
                    emit_py_expr(expr),
                    py_ident(al)
                )
            } else {
                format!("{}with {}:\n", pad, emit_py_expr(expr))
            };
            s.push_str(&emit_py_body(body, indent + 1, in_init));
            s
        }
        Stmt::Function(func) => emit_py_function(func, indent, false),
        Stmt::Class(cls) => emit_py_class(cls, indent),
        Stmt::Return(Some(expr)) => {
            if in_init {
                format!("{}return\n", pad)
            } else {
                format!("{}return {}\n", pad, emit_py_expr(expr))
            }
        }
        Stmt::Return(None) => format!("{}return\n", pad),
        Stmt::Break => format!("{}break\n", pad),
        Stmt::Continue => format!("{}continue\n", pad),
        Stmt::TryCatch {
            try_body,
            catch_var,
            catch_body,
            finally_body,
        } => {
            let mut s = format!("{}try:\n", pad);
            s.push_str(&emit_py_body(try_body, indent + 1, in_init));
            if let Some(var) = catch_var {
                s.push_str(&format!("{}except Exception as {}:\n", pad, py_ident(var)));
            } else {
                s.push_str(&format!("{}except Exception:\n", pad));
            }
            s.push_str(&emit_py_body(catch_body, indent + 1, in_init));
            if let Some(fin) = finally_body {
                s.push_str(&format!("{}finally:\n", pad));
                s.push_str(&emit_py_body(fin, indent + 1, in_init));
            }
            s
        }
        Stmt::Throw(expr) => {
            format!("{}raise RuntimeError({})\n", pad, emit_py_expr(expr))
        }
        Stmt::Match {
            target,
            cases,
            default_case,
        } => {
            let mut s = format!("{}_lsn_match = {}\n", pad, emit_py_expr(target));
            if cases.is_empty() {
                if let Some(def_stmts) = default_case {
                    s.push_str(&emit_py_body(def_stmts, indent, in_init));
                }
                return s;
            }
            for (idx, (case_expr, case_body)) in cases.iter().enumerate() {
                let kw = if idx == 0 { "if" } else { "elif" };
                s.push_str(&format!(
                    "{}{} _lsn_match == {}:\n",
                    pad,
                    kw,
                    emit_py_expr(case_expr)
                ));
                s.push_str(&emit_py_body(case_body, indent + 1, in_init));
            }
            if let Some(def_stmts) = default_case {
                s.push_str(&format!("{}else:\n", pad));
                s.push_str(&emit_py_body(def_stmts, indent + 1, in_init));
            }
            s
        }
    }
}

fn emit_py_if(
    cond: &Expr,
    yes: &[Stmt],
    no: Option<&[Stmt]>,
    indent: usize,
    in_init: bool,
    kw: &str,
) -> String {
    let pad = "    ".repeat(indent);
    let mut s = format!("{}{} {}:\n", pad, kw, emit_py_expr(cond));
    s.push_str(&emit_py_body(yes, indent + 1, in_init));
    if let Some(no_body) = no {
        if no_body.len() == 1 {
            if let Stmt::If(elif_cond, elif_yes, elif_no) = no_body[0].unwrap_spanned() {
                s.push_str(&emit_py_if(
                    elif_cond,
                    elif_yes,
                    elif_no.as_deref(),
                    indent,
                    in_init,
                    "elif",
                ));
                return s;
            }
        }
        s.push_str(&format!("{}else:\n", pad));
        s.push_str(&emit_py_body(no_body, indent + 1, in_init));
    }
    s
}

fn emit_py_function(func: &Function, indent: usize, is_method: bool) -> String {
    let pad = "    ".repeat(indent);
    let is_init = is_method && (func.name == "بناء" || func.name == "تهيئة");
    let fn_name = if is_init {
        "__init__".to_string()
    } else {
        py_ident(&func.name)
    };

    let mut params = Vec::new();
    if is_method {
        params.push("self".to_string());
    }
    for (idx, param) in func.params.iter().enumerate() {
        let p_ident = py_ident(param);
        if let Some(Some(def_expr)) = func.defaults.get(idx) {
            params.push(format!("{}={}", p_ident, emit_py_expr(def_expr)));
        } else {
            params.push(p_ident);
        }
    }
    if let Some(vararg) = &func.vararg {
        params.push(format!("*{}", py_ident(vararg)));
    }

    let mut s = format!("{}def {}({}):\n", pad, fn_name, params.join(", "));
    s.push_str(&emit_py_body(&func.body, indent + 1, is_init));
    s.push('\n');
    s
}

fn emit_py_class(cls: &ClassDef, indent: usize) -> String {
    let pad = "    ".repeat(indent);
    let cname = py_ident(&cls.name);
    let mut s = if let Some(parent) = &cls.parent {
        format!("{}class {}({}):\n", pad, cname, py_ident(parent))
    } else {
        format!("{}class {}:\n", pad, cname)
    };

    if cls.methods.is_empty() {
        s.push_str(&format!("{}    pass\n\n", pad));
    } else {
        for m in &cls.methods {
            s.push_str(&emit_py_function(m, indent + 1, true));
        }
    }
    s
}

fn is_known_str_expr(expr: &Expr) -> bool {
    match expr.unwrap_spanned() {
        Expr::Text(_) | Expr::FString(_) => true,
        Expr::Call(name, _) if name == "نص" => true,
        Expr::Binary(l, Op::Add, r) => is_known_str_expr(l) || is_known_str_expr(r),
        _ => false,
    }
}

fn emit_py_expr(expr: &Expr) -> String {
    match expr {
        Expr::Spanned(inner, _, _) => emit_py_expr(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Text(t) => py_string_literal(t),
        Expr::Bool(true) => "True".into(),
        Expr::Bool(false) => "False".into(),
        Expr::Null => "None".into(),
        Expr::List(items) => {
            let vals: Vec<String> = items.iter().map(emit_py_expr).collect();
            format!("[{}]", vals.join(", "))
        }
        Expr::Map(pairs) => {
            let entries: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", py_string_literal(k), emit_py_expr(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        Expr::Variable(name) => match name.as_str() {
            "هذا" | "ذاتي" => "self".into(),
            "ط" | "قيمة_ط" => "math.pi".into(),
            "هـ" | "قيمة_هـ" => "math.e".into(),
            other => py_ident(other),
        },
        Expr::Assign(name, val) => {
            format!("({} := {})", py_ident(name), emit_py_expr(val))
        }
        Expr::AssignIndex(name, idx, val) => {
            format!(
                "{}.__setitem__({}, {})",
                py_ident(name),
                emit_py_expr(idx),
                emit_py_expr(val)
            )
        }
        Expr::Index(target, idx) => {
            format!("{}[{}]", emit_py_expr(target), emit_py_expr(idx))
        }
        Expr::Unary(op, inner) => {
            let v = emit_py_expr(inner);
            match op {
                Op::Negate => format!("(-{})", v),
                Op::Not => format!("(not {})", v),
                _ => v,
            }
        }
        Expr::Binary(left, op, right) => {
            let l = emit_py_expr(left);
            let r = emit_py_expr(right);
            if matches!(op, Op::Add) {
                let l_str = is_known_str_expr(left);
                let r_str = is_known_str_expr(right);
                if l_str && r_str {
                    return format!("({} + {})", l, r);
                } else if l_str {
                    return format!("({} + str({}))", l, r);
                } else if r_str {
                    return format!("(str({}) + {})", l, r);
                } else {
                    return format!("_lsn_add({}, {})", l, r);
                }
            }
            let op_str = match op {
                Op::Add => "+",
                Op::Subtract => "-",
                Op::Multiply => "*",
                Op::Divide => "/",
                Op::Modulo => "%",
                Op::Power => "**",
                Op::Equal => "==",
                Op::NotEqual => "!=",
                Op::Less => "<",
                Op::LessEqual => "<=",
                Op::Greater => ">",
                Op::GreaterEqual => ">=",
                Op::And => "and",
                Op::Or => "or",
                Op::In => "in",
                Op::NotIn => "not in",
                Op::Not | Op::Negate => "==",
            };
            format!("({} {} {})", l, op_str, r)
        }
        Expr::Call(name, args) => emit_py_call(name, args),
        Expr::GetProperty(target, prop) => {
            format!("{}.{}", emit_py_expr(target), py_ident(prop))
        }
        Expr::SetProperty(target, prop, val) => {
            format!(
                "setattr({}, {}, {})",
                emit_py_expr(target),
                py_string_literal(prop),
                emit_py_expr(val)
            )
        }
        Expr::MethodCall(target, method, args) => emit_py_method_call(target, method, args),
        Expr::Lambda(params, body) => {
            let py_params: Vec<String> = params.iter().map(|p| py_ident(p)).collect();
            if body.len() == 1 {
                if let Stmt::Return(Some(ret_expr)) = body[0].unwrap_spanned() {
                    return format!("(lambda {}: {})", py_params.join(", "), emit_py_expr(ret_expr));
                }
            }
            format!("(lambda {}: None)", py_params.join(", "))
        }
        Expr::Slice(target, start, end, step) => {
            let t = emit_py_expr(target);
            let s = start.as_ref().map(|e| emit_py_int_slice(e)).unwrap_or_default();
            let e = end.as_ref().map(|e| emit_py_int_slice(e)).unwrap_or_default();
            if let Some(st) = step {
                format!("{}[{}:{}:{}]", t, s, e, emit_py_int_slice(st))
            } else {
                format!("{}[{}:{}]", t, s, e)
            }
        }
        Expr::ListComprehension {
            expr,
            var,
            iterable,
            condition,
        } => {
            let e = emit_py_expr(expr);
            let v = py_ident(var);
            let it = emit_py_expr(iterable);
            if let Some(cond) = condition {
                format!("[{} for {} in {} if {}]", e, v, it, emit_py_expr(cond))
            } else {
                format!("[{} for {} in {}]", e, v, it)
            }
        }
        Expr::AssignUnpack(names, val) => {
            let lhs: Vec<String> = names.iter().map(|n| py_ident(n)).collect();
            format!("({} := {})", lhs.join(", "), emit_py_expr(val))
        }
        Expr::FString(segments) => emit_py_fstring(segments),
    }
}

fn emit_py_int_slice(expr: &Expr) -> String {
    match expr.unwrap_spanned() {
        Expr::Number(n) if n.fract() == 0.0 => format!("{}", *n as i64),
        other => format!("int({})", emit_py_expr(other)),
    }
}

fn emit_py_fstring(segments: &[Expr]) -> String {
    if segments.is_empty() {
        return "\"\"".into();
    }
    let parts: Vec<String> = segments
        .iter()
        .map(|seg| match seg.unwrap_spanned() {
            Expr::Text(t) => py_string_literal(t),
            other => format!("str({})", emit_py_expr(other)),
        })
        .collect();
    if parts.len() == 1 {
        parts[0].clone()
    } else {
        format!("({})", parts.join(" + "))
    }
}

fn emit_py_call(name: &str, args: &[Expr]) -> String {
    let py_args: Vec<String> = args.iter().map(emit_py_expr).collect();
    match name {
        "نص" if args.len() == 1 => format!("str({})", py_args[0]),
        "عدد" if args.len() == 1 => format!("float({})", py_args[0]),
        "عدد_صحيح" if args.len() == 1 => format!("int({})", py_args[0]),
        "طول" if args.len() == 1 => format!("len({})", py_args[0]),
        "نوع" if args.len() == 1 => format!("type({}).__name__", py_args[0]),
        "مدى" | "نطاق" if !args.is_empty() && args.len() <= 3 => {
            let int_args: Vec<String> = args.iter().map(emit_py_int_slice).collect();
            format!("range({})", int_args.join(", "))
        }
        "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 2 => {
            format!("{}.append({})", py_args[0], py_args[1])
        }
        "مطلق" if args.len() == 1 => format!("abs({})", py_args[0]),
        "جذر" if args.len() == 1 => format!("math.sqrt({})", py_args[0]),
        "قرّب" | "قرب" if args.len() == 1 => format!("round({})", py_args[0]),
        "قرّب" | "قرب" if args.len() == 2 => format!("round({}, int({}))", py_args[0], py_args[1]),
        "أرضية" | "ارضية" if args.len() == 1 => format!("math.floor({})", py_args[0]),
        "سقف" if args.len() == 1 => format!("math.ceil({})", py_args[0]),
        "قوة" if args.len() == 2 => format!("pow({}, {})", py_args[0], py_args[1]),
        "أدنى" | "ادنى" if !args.is_empty() => format!("min({})", py_args.join(", ")),
        "أعلى" | "اعلى" if !args.is_empty() => format!("max({})", py_args.join(", ")),
        "مجموع" if args.len() == 1 => format!("sum({})", py_args[0]),
        "متوسط" if args.len() == 1 => format!("(sum({}) / len({}))", py_args[0], py_args[0]),
        "فرز" if args.len() == 1 => format!("sorted({})", py_args[0]),
        "عكس" | "اعكس" if args.len() == 1 => format!("list(reversed({}))", py_args[0]),
        "انتظر_ملي" | "تأخير" | "تاخير" if args.len() == 1 => {
            format!("time.sleep(({}) / 1000.0)", py_args[0])
        }
        "انتظر_ميكرو" if args.len() == 1 => {
            format!("time.sleep(({}) / 1000000.0)", py_args[0])
        }
        "وقت_التشغيل" if args.is_empty() => "int(time.monotonic() * 1000)".into(),
        "طابع_زمني" | "الآن" | "الان" if args.is_empty() => "time.time()".into(),
        "خريطة_قيمة" if args.len() == 5 => format!(
            "((({} - {}) * ({} - {}) / ({} - {})) + {})",
            py_args[0], py_args[1], py_args[4], py_args[3], py_args[2], py_args[1], py_args[3]
        ),
        "قيد_قيمة" if args.len() == 3 => {
            format!("max({}, min({}, {}))", py_args[1], py_args[2], py_args[0])
        }
        other => format!("{}({})", py_ident(other), py_args.join(", ")),
    }
}

fn emit_py_method_call(target: &Expr, method: &str, args: &[Expr]) -> String {
    let t = emit_py_expr(target);
    let py_args: Vec<String> = args.iter().map(emit_py_expr).collect();
    match method {
        "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 1 => {
            format!("{}.append({})", t, py_args[0])
        }
        "طول" if args.is_empty() => format!("len({})", t),
        "مسح" if args.is_empty() => format!("{}.clear()", t),
        "نسخ" if args.is_empty() => format!("{}.copy()", t),
        "مجموع" if args.is_empty() => format!("sum({})", t),
        "تحتوي" | "يحتوي" if args.len() == 1 => format!("({} in {})", py_args[0], t),
        "تقسيم" if args.len() == 1 => format!("{}.split({})", t, py_args[0]),
        "أحرف_كبيرة" | "احرف_كبيرة" if args.is_empty() => format!("{}.upper()", t),
        "أحرف_صغيرة" | "احرف_صغيرة" if args.is_empty() => format!("{}.lower()", t),
        "استبدل" | "أعد" if args.len() == 2 => {
            format!("{}.replace({}, {})", t, py_args[0], py_args[1])
        }
        other => format!("{}.{}({})", t, py_ident(other), py_args.join(", ")),
    }
}

fn py_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn py_ident(name: &str) -> String {
    if is_python_keyword(name) {
        return format!("{}_", name);
    }
    let mut out = String::with_capacity(name.len());
    for (idx, c) in name.chars().enumerate() {
        let valid = if idx == 0 {
            c == '_' || c.is_alphabetic()
        } else {
            c == '_'
                || c.is_alphanumeric()
                || ('\u{064B}'..='\u{065F}').contains(&c)
                || c == '\u{0670}'
                || c == '\u{0640}'
        };
        if valid {
            out.push(c);
        } else {
            if idx == 0 {
                out.push('_');
            }
            out.push_str(&format!("_u{:x}_", c as u32));
        }
    }
    if out.is_empty() {
        "_lsn_id".into()
    } else {
        out
    }
}

fn is_python_keyword(name: &str) -> bool {
    matches!(
        name,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_raspberry_parses_and_simulates_all_classes() {
        let user_code = r#"
دع لوحة = جديد لوحة_راسبيري("Raspberry Pi 5")
تحقق_يساوي(لوحة.اسم_الطراز()، "Raspberry Pi 5")
تحقق_يساوي(لوحة.عدد_الأنوية()، 4)

دع منفذ = جديد منفذ_gpio_راسبيري(17، "مخرج")
منفذ.شغّل()
تحقق_يساوي(منفذ.اقرأ()، 1)
منفذ.اقلب()
تحقق_يساوي(منفذ.اقرأ()، 0)
منفذ.تنظيف()

دع نبضة = جديد نبضة_pwm_راسبيري(18، 1000)
نبضة.ابدأ(75)
تحقق_يساوي(نبضة.غير_النسبة(60)، 60)
نبضة.أوقف()

دع ناقل = جديد ناقل_i2c_راسبيري(1)
ناقل.اكتب_بايت(64، 10، 123)
تحقق_يساوي(ناقل.اقرأ_بايت(64، 10)، 123)
تحقق(ناقل.فحص_الأجهزة().طول() >= 3)

دع سبي = جديد ناقل_spi_راسبيري(0، 0، 1000000)
دع رد_سبي = سبي.نقل([1، 2، 3])
تحقق_يساوي(رد_سبي.طول()، 3)
سبي.أغلق()

دع تسلسلي = جديد منفذ_تسلسلي_راسبيري("/dev/serial0"، 115200)
تسلسلي.اكتب_سطر("AT")
تحقق_يساوي(تسلسلي.اقرأ_سطر()، "OK")
تسلسلي.أغلق()

دع كاميرا = جديد كاميرا_راسبيري(1920، 1080)
كاميرا.ابدأ()
تحقق(كاميرا.التقط_صورة("/tmp/frame.jpg"))
تحقق(كاميرا.سجل_فيديو("/tmp/clip.mp4"، 3))
تحقق(كاميرا.ابدأ_بث(8080))
كاميرا.أوقف()

دع حساسات = جديد قبعة_الحساسات_sensehat()
حساسات.اعرض_رسالة("مرحبا"، 0.08)
حساسات.اضبط_بكسل(0، 0، 0، 255، 0)
تحقق(حساسات.اقرأ_حرارة() > 0)
تحقق(حساسات.اقرأ_رطوبة() > 0)
تحقق(حساسات.اقرأ_ضغط() > 900)
دع توجيه = حساسات.اقرأ_توجيه()
تحقق("انحدار" في توجيه)
حساسات.امسح_الشاشة()

دع محركات = جديد قبعة_محركات_pca9685(64)
تحقق_يساوي(محركات.اضبط_زاوية_سيرفو(0، 90)، 90)
تحقق_يساوي(محركات.اضبط_سرعة_محرك(1، 85)، 85)

دع شاشة_لمس = جديد شاشة_لمس_dsi(7)
تحقق_يساوي(شاشة_لمس.اضبط_سطوع(90)، 90)
تحقق_يساوي(شاشة_لمس.شغّل_الشاشة(صحيح)، صحيح)

دع صوت = جديد صوت_راسبيري(80)
تحقق_يساوي(صوت.اضبط_الصوت(70)، 70)
تحقق(صوت.شغّل_ملف("alert.wav"))
تحقق(صوت.انطق_عربي("النظام جاهز"))

دع حبر = جديد شاشة_حبر_إلكتروني_epaper(250، 122)
حبر.امسح()
حبر.اعرض_نص(10، 20، "راسبيري باي 5")
حبر.تحديث_الشاشة()
حبر.سكون()
"#;
        let combined = format!("{}\n{}", STDLIB_RASPBERRY, user_code);
        let result = crate::Interpreter::default().run_source(&combined);
        assert!(
            result.is_ok(),
            "فشل تنفيذ مكتبة راسبيري القياسية: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_transpile_raspberry_python_output() {
        let code = r#"
استورد "راسبيري"
دع لوحة = جديد لوحة_راسبيري("Raspberry Pi 5")
اطبع("الطراز: " + لوحة.اسم_الطراز())
دع مصباح = جديد منفذ_gpio_راسبيري(17، "مخرج")
لكل ع في مدى(3) {
    إذا (ع % 2 == 0) {
        مصباح.شغّل()
    } وإلا {
        مصباح.أوقف()
    }
}
مصباح.تنظيف()
"#;
        let py = transpile_raspberry_python(code).expect("يجب أن تنجح ترجمة راسبيري باي إلى بايثون 3");
        assert!(py.starts_with("#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n"));
        assert!(py.contains("import RPi.GPIO"));
        assert!(py.contains("import gpiozero"));
        assert!(py.contains("import smbus2"));
        assert!(py.contains("import spidev"));
        assert!(py.contains("import serial"));
        assert!(py.contains("from picamera2 import Picamera2"));
        assert!(py.contains("from sense_hat import SenseHat"));
        assert!(py.contains("لوحة = لوحة_راسبيري(\"Raspberry Pi 5\")"));
        assert!(py.contains("for ع in range(3):"));
    }

    #[test]
    fn test_build_raspberry_file_creates_executable_py() {
        let tmp_dir = std::env::temp_dir().join(format!("lisan_rpi_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp_dir);
        let in_file = tmp_dir.join("blink.lsn");
        std::fs::write(
            &in_file,
            "استورد \"راسبيري\"\nدع م = جديد منفذ_gpio_راسبيري(18)\nم.شغّل()\nم.تنظيف()\n",
        )
        .unwrap();

        let out_file = build_raspberry_file(&in_file, None).expect("يجب إنشاء ملف بايثون راسبيري");
        assert_eq!(out_file.file_name().unwrap().to_str().unwrap(), "blink_rpi.py");
        assert!(out_file.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&out_file).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111);
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}

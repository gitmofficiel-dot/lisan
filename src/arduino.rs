//! مترجم ومحاكي الأردوينو والمتحكمات الدقيقة (Arduino / ESP32 / STM32) للغة لسان.
use crate::ast::{ClassDef, Expr, Function, Op, Stmt};
use std::path::{Path, PathBuf};

/// المكتبة القياسية المدمجة للأردوينو (Arduino AVR / Uno / Nano / Mega)
pub const STDLIB_ARDUINO: &str = r#"
# مكتبة الأردوينو القياسية في لغة لسان (Arduino Uno / Nano / Mega)

دع مخرج = "OUTPUT"
دع مدخل = "INPUT"
دع مدخل_سحب = "INPUT_PULLUP"
دع عالي = 1
دع منخفض = 0

صنف محرك_سيرفو {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.الزاوية = 90
        وضع_المنفذ(منفذ، "OUTPUT")
    }

    دالة حرك(زاوية) {
        إذا (زاوية < 0) { زاوية = 0 }
        إذا (زاوية > 180) { زاوية = 180 }
        هذا.الزاوية = زاوية
        اكتب_تماثلي(هذا.منفذ، زاوية)
        أرجع هذا.الزاوية
    }

    دالة اقرأ_زاوية() {
        أرجع هذا.الزاوية
    }
}

صنف محرك_خطوي {
    دالة بناء(خطوات_في_الدورة، منفذ1، منفذ2، منفذ3، منفذ4) {
        هذا.خطوات = خطوات_في_الدورة
        هذا.منفذ1 = منفذ1
        هذا.منفذ2 = منفذ2
        هذا.منفذ3 = منفذ3
        هذا.منفذ4 = منفذ4
        هذا.سرعة_دقيقة = 60
        هذا.الموقع_الحالي = 0
    }

    دالة اضبط_السرعة(لفة_بالدقيقة) {
        هذا.سرعة_دقيقة = لفة_بالدقيقة
        أرجع لفة_بالدقيقة
    }

    دالة حرك(خطوات) {
        هذا.الموقع_الحالي = هذا.الموقع_الحالي + خطوات
        أرجع هذا.الموقع_الحالي
    }

    دالة الموقع() {
        أرجع هذا.الموقع_الحالي
    }
}

صنف حساس_مسافة {
    دالة بناء(منفذ_إرسال، منفذ_استقبال) {
        هذا.إرسال = منفذ_إرسال
        هذا.استقبال = منفذ_استقبال
        هذا.آخر_قراءة = 25
        وضع_المنفذ(منفذ_إرسال، "OUTPUT")
        وضع_المنفذ(منفذ_استقبال، "INPUT")
    }

    دالة اقرأ_سم() {
        اكتب_رقمي(هذا.إرسال، 0)
        انتظر_ميكرو(2)
        اكتب_رقمي(هذا.إرسال، 1)
        انتظر_ميكرو(10)
        اكتب_رقمي(هذا.إرسال، 0)
        أرجع هذا.آخر_قراءة
    }

    دالة ضبط_محاكاة(مسافة_سم) {
        هذا.آخر_قراءة = مسافة_سم
        أرجع مسافة_سم
    }

    دالة هل_قريب(حد_سم) {
        أرجع هذا.اقرأ_سم() <= حد_سم
    }
}

صنف حساس_حرارة {
    دالة بناء(منفذ، نوع_الحساس) {
        هذا.منفذ = منفذ
        هذا.نوع_الحساس = نوع_الحساس
        هذا.حرارة = 26.5
        هذا.رطوبة = 55.0
        وضع_المنفذ(منفذ، "INPUT")
    }

    دالة اقرأ_حرارة() {
        أرجع هذا.حرارة
    }

    دالة اقرأ_رطوبة() {
        أرجع هذا.رطوبة
    }

    دالة ضبط_محاكاة(حرارة_جديدة، رطوبة_جديدة) {
        هذا.حرارة = حرارة_جديدة
        هذا.رطوبة = رطوبة_جديدة
    }
}

صنف شاشة_كريستال {
    دالة بناء(أعمدة، صفوف) {
        هذا.أعمدة = أعمدة
        هذا.صفوف = صفوف
        هذا.سطر0 = ""
        هذا.سطر1 = ""
        هذا.إضاءة_خلفية = صحيح
    }

    دالة امسح() {
        هذا.سطر0 = ""
        هذا.سطر1 = ""
        أرجع صحيح
    }

    دالة اطبع_في(عمود، صف، نص_الشاشة) {
        إذا (صف == 0) {
            هذا.سطر0 = نص(نص_الشاشة)
        } وإلا {
            هذا.سطر1 = نص(نص_الشاشة)
        }
        أرجع نص_الشاشة
    }

    دالة محتوى_السطر(صف) {
        إذا (صف == 0) {
            أرجع هذا.سطر0
        }
        أرجع هذا.سطر1
    }
}

صنف شريط_إضاءة {
    دالة بناء(منفذ، عدد_المصابيح) {
        هذا.منفذ = منفذ
        هذا.عدد = عدد_المصابيح
        هذا.ألوان = []
        وضع_المنفذ(منفذ، "OUTPUT")
    }

    دالة لون_مصباح(فهرس، أحمر، أخضر، أزرق) {
        أرجع [فهرس، أحمر، أخضر، أزرق]
    }

    دالة اعرض() {
        أرجع صحيح
    }
}

صنف ذاكرة_eeprom {
    دالة بناء() {
        هذا.بيانات = {}
    }

    دالة اقرأ(عنوان) {
        دع مفتاح = نص(عنوان)
        إذا (مفتاح في هذا.بيانات) {
            أرجع هذا.بيانات[مفتاح]
        }
        أرجع 255
    }

    دالة اكتب(عنوان، قيمة) {
        دع مفتاح = نص(عنوان)
        دع ب = هذا.بيانات
        ب[مفتاح] = قيمة
        أرجع قيمة
    }

    دالة احفظ() {
        أرجع صحيح
    }
}

صنف ناقل_i2c {
    دالة بناء() {
        هذا.متصل = صحيح
        هذا.العنوان = 0
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة ابدأ_إرسال(عنوان_الجهاز) {
        هذا.العنوان = عنوان_الجهاز
        أرجع صحيح
    }

    دالة أرسل_بايت(بايت) {
        أرجع بايت
    }

    دالة أنهِ_إرسال() {
        أرجع 0
    }

    دالة اطلب_بيانات(عنوان_الجهاز، عدد_البايتات) {
        أرجع عدد_البايتات
    }
}

صنف زر_ضاغط {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.حالة_سابقة = خطأ
        وضع_المنفذ(منفذ، "INPUT_PULLUP")
    }

    دالة هل_مضغوط() {
        دع قراءة = اقرأ_رقمي(هذا.منفذ)
        أرجع قراءة == 0
    }

    دالة نقرة() {
        دع مضغوط = هذا.هل_مضغوط()
        إذا (مضغوط && !هذا.حالة_سابقة) {
            هذا.حالة_سابقة = صحيح
            أرجع صحيح
        }
        إذا (!مضغوط) {
            هذا.حالة_سابقة = خطأ
        }
        أرجع خطأ
    }
}

صنف جرس_تنبيه {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        وضع_المنفذ(منفذ، "OUTPUT")
    }

    دالة نغمة(تردد، مدة_ملي = 200) {
        نبضة_صوت(هذا.منفذ، تردد، مدة_ملي)
        أرجع صحيح
    }

    دالة أوقف() {
        أوقف_صوت(هذا.منفذ)
        أرجع صحيح
    }
}

صنف متحكم_محركات_l298n {
    دالة بناء(منفذ_تفعيل_أ، منفذ1، منفذ2، منفذ_تفعيل_ب، منفذ3، منفذ4) {
        هذا.تفعيل_أ = منفذ_تفعيل_أ
        هذا.منفذ1 = منفذ1
        هذا.منفذ2 = منفذ2
        هذا.تفعيل_ب = منفذ_تفعيل_ب
        هذا.منفذ3 = منفذ3
        هذا.منفذ4 = منفذ4
        وضع_المنفذ(منفذ_تفعيل_أ، "OUTPUT")
        وضع_المنفذ(منفذ1، "OUTPUT")
        وضع_المنفذ(منفذ2، "OUTPUT")
        وضع_المنفذ(منفذ_تفعيل_ب، "OUTPUT")
        وضع_المنفذ(منفذ3، "OUTPUT")
        وضع_المنفذ(منفذ4، "OUTPUT")
    }

    دالة تحريك_أمام(سرعة = 200) {
        اكتب_تماثلي(هذا.تفعيل_أ، سرعة)
        اكتب_رقمي(هذا.منفذ1، 1)
        اكتب_رقمي(هذا.منفذ2، 0)
        اكتب_تماثلي(هذا.تفعيل_ب، سرعة)
        اكتب_رقمي(هذا.منفذ3، 1)
        اكتب_رقمي(هذا.منفذ4، 0)
        أرجع صحيح
    }

    دالة تحريك_خلف(سرعة = 200) {
        اكتب_تماثلي(هذا.تفعيل_أ، سرعة)
        اكتب_رقمي(هذا.منفذ1، 0)
        اكتب_رقمي(هذا.منفذ2، 1)
        اكتب_تماثلي(هذا.تفعيل_ب، سرعة)
        اكتب_رقمي(هذا.منفذ3، 0)
        اكتب_رقمي(هذا.منفذ4، 1)
        أرجع صحيح
    }

    دالة إيقاف() {
        اكتب_تماثلي(هذا.تفعيل_أ، 0)
        اكتب_تماثلي(هذا.تفعيل_ب، 0)
        أرجع صحيح
    }

    دالة أوقف() {
        أرجع هذا.إيقاف()
    }
}

صنف واي_فاي_متحكم {
    دالة بناء() {
        هذا.الشبكة = ""
        هذا.متصل = خطأ
        هذا.عنوان = "0.0.0.0"
    }

    دالة اتصل(اسم_الشبكة، كلمة_المرور) {
        هذا.الشبكة = اسم_الشبكة
        هذا.متصل = صحيح
        هذا.عنوان = "192.168.1.105"
        أرجع صحيح
    }

    دالة هل_متصل() {
        أرجع هذا.متصل
    }

    دالة عنوان_ip() {
        أرجع هذا.عنوان
    }
}

صنف بلوتوث_متحكم {
    دالة بناء(اسم_الجهاز) {
        هذا.الاسم = اسم_الجهاز
        هذا.متصل = خطأ
    }

    دالة ابدأ() {
        هذا.متصل = صحيح
        أرجع صحيح
    }

    دالة أرسل(بيانات) {
        أرجع صحيح
    }

    دالة متاح() {
        أرجع خطأ
    }
}

صنف شاشة_oled {
    دالة بناء(عنوان = 60، عرض = 128، ارتفاع = 64) {
        هذا.العنوان = عنوان
        هذا.العرض = عرض
        هذا.الارتفاع = ارتفاع
        هذا.نصوص = []
        هذا.مفعل = صحيح
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة امسح() {
        هذا.نصوص = []
        أرجع صحيح
    }

    دالة اكتب_نص(س، ص، رسالة، حجم = 1) {
        هذا.نصوص.أضف({"س": س، "ص": ص، "نص": نص(رسالة)، "حجم": حجم})
        أرجع صحيح
    }

    دالة ارسم_خط(س1، ص1، س2، ص2) {
        أرجع صحيح
    }

    دالة ارسم_مستطيل(س، ص، عرض، ارتفاع، ممتلئ = خطأ) {
        أرجع صحيح
    }

    دالة ارسم_دائرة(س، ص، نصف_القطر، ممتلئ = خطأ) {
        أرجع صحيح
    }

    دالة شريط_تقدم(س، ص، عرض، ارتفاع، نسبة) {
        أرجع صحيح
    }

    دالة اعرض() {
        أرجع صحيح
    }
}

صنف مرحل_كهربائي {
    دالة بناء(منفذ، عكسي = خطأ) {
        هذا.منفذ = منفذ
        هذا.عكسي = عكسي
        هذا.مغلق = خطأ
        وضع_المنفذ(منفذ، "OUTPUT")
        هذا.أوقف()
    }

    دالة شغّل() {
        هذا.مغلق = صحيح
        دع قيمة = 1
        إذا (هذا.عكسي) {
            قيمة = 0
        }
        اكتب_رقمي(هذا.منفذ، قيمة)
        أرجع صحيح
    }

    دالة شتغل() {
        أرجع هذا.شغّل()
    }

    دالة أوقف() {
        هذا.مغلق = خطأ
        دع قيمة = 0
        إذا (هذا.عكسي) {
            قيمة = 1
        }
        اكتب_رقمي(هذا.منفذ، قيمة)
        أرجع صحيح
    }

    دالة اقلب() {
        إذا (هذا.مغلق) {
            أرجع هذا.أوقف()
        } وإلا {
            أرجع هذا.شغّل()
        }
    }

    دالة هل_يعمل() {
        أرجع هذا.مغلق
    }
}

صنف حساس_حركة_pir {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.محاكاة_حركة = خطأ
        وضع_المنفذ(منفذ، "INPUT")
    }

    دالة ضبط_محاكاة(يوجد_حركة) {
        هذا.محاكاة_حركة = يوجد_حركة
        أرجع يوجد_حركة
    }

    دالة هل_يوجد_حركة() {
        إذا (هذا.محاكاة_حركة) {
            أرجع صحيح
        }
        دع قراءة = اقرأ_رقمي(هذا.منفذ)
        أرجع قراءة == 1
    }
}

صنف حساس_غاز_mq {
    دالة بناء(منفذ_تماثلي، منفذ_رقمي = -1) {
        هذا.منفذ_ت = منفذ_تماثلي
        هذا.منفذ_ر = منفذ_رقمي
        هذا.محاكاة_ppm = 120
        وضع_المنفذ(منفذ_تماثلي، "INPUT")
        إذا (منفذ_رقمي >= 0) {
            وضع_المنفذ(منفذ_رقمي، "INPUT")
        }
    }

    دالة ضبط_محاكاة(قيمة_ppm) {
        هذا.محاكاة_ppm = قيمة_ppm
        أرجع قيمة_ppm
    }

    دالة اقرأ_كثافة_ppm() {
        أرجع هذا.محاكاة_ppm
    }

    دالة هل_يوجد_تسريب(حد_ppm = 400) {
        أرجع هذا.اقرأ_كثافة_ppm() > حد_ppm
    }
}

صنف محدد_مواقع_gps {
    دالة بناء(منفذ_rx، منفذ_tx، سرعة = 9600) {
        هذا.rx = منفذ_rx
        هذا.tx = منفذ_tx
        هذا.سرعة = سرعة
        هذا.خط_العرض = 24.7136
        هذا.خط_الطول = 46.6753
        هذا.الارتفاع = 612.0
        هذا.السرعة_كم = 45.0
        هذا.عدد_الأقمار = 9
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة حدّث() {
        أرجع صحيح
    }

    دالة خط_العرض() {
        أرجع هذا.خط_العرض
    }

    دالة خط_الطول() {
        أرجع هذا.خط_الطول
    }

    دالة الارتفاع() {
        أرجع هذا.الارتفاع
    }

    دالة السرعة() {
        أرجع هذا.السرعة_كم
    }

    دالة الأقمار() {
        أرجع هذا.عدد_الأقمار
    }

    دالة ضبط_محاكاة(عرض، طول، سرعة = 0.0) {
        هذا.خط_العرض = عرض
        هذا.خط_الطول = طول
        هذا.السرعة_كم = سرعة
        أرجع صحيح
    }
}
"#;

/// المكتبة القياسية المدمجة لمتحكمات ESP32 و ESP8266
pub const STDLIB_ESP32: &str = r#"
# مكتبة متحكمات ESP32 و ESP8266 في لغة لسان (WiFi / BLE / WebServer / MQTT)

صنف واي_فاي_esp {
    دالة بناء() {
        هذا.متصل = خطأ
        هذا.اسم_الشبكة = ""
        هذا.ip = "0.0.0.0"
        هذا.قوة_الإشارة = -65
    }

    دالة اتصل(اسم_الشبكة، كلمة_المرور) {
        هذا.اسم_الشبكة = اسم_الشبكة
        هذا.متصل = صحيح
        هذا.ip = "192.168.1.150"
        تسلسلي_اطبع("تم الاتصال بشبكة WiFi: " + اسم_الشبكة)
        أرجع صحيح
    }

    دالة نقطة_وصول(اسم_الشبكة، كلمة_المرور = "") {
        هذا.اسم_الشبكة = اسم_الشبكة
        هذا.متصل = صحيح
        هذا.ip = "192.168.4.1"
        تسلسلي_اطبع("تم تفعيل نقطة الوصول AP: " + اسم_الشبكة)
        أرجع صحيح
    }

    دالة هل_متصل() {
        أرجع هذا.متصل
    }

    دالة عنوان_ip() {
        أرجع هذا.ip
    }

    دالة قوة_الاشارة() {
        أرجع هذا.قوة_الإشارة
    }

    دالة مسح_الشبكات() {
        أرجع ["Lisan_Home_WiFi", "IoT_Smart_Hub", "ESP32_Device"]
    }
}

صنف خادم_ويب_esp {
    دالة بناء(منفذ = 80) {
        هذا.منفذ = منفذ
        هذا.مسارات = {}
        هذا.يعمل = خطأ
    }

    دالة عند_طلب(مسار، رد_نصي) {
        دع م = هذا.مسارات
        م[مسار] = رد_نصي
        أرجع صحيح
    }

    دالة ابدأ() {
        هذا.يعمل = صحيح
        تسلسلي_اطبع("بدأ تشغيل خادم الويب على المنفذ " + نص(هذا.منفذ))
        أرجع صحيح
    }

    دالة تعامل_مع_العملاء() {
        أرجع صحيح
    }
}

صنف عميل_mqtt {
    دالة بناء(عنوان_الوسيط، منفذ = 1883) {
        هذا.وسيط = عنوان_الوسيط
        هذا.منفذ = منفذ
        هذا.متصل = خطأ
        هذا.اشتراكات = []
    }

    دالة اتصل(معرف_العميل) {
        هذا.متصل = صحيح
        تسلسلي_اطبع("تم الاتصال بوسيط MQTT: " + هذا.وسيط)
        أرجع صحيح
    }

    دالة انشر(الموضوع، الرسالة) {
        تسلسلي_اطبع("[MQTT انشر] " + الموضوع + " -> " + نص(الرسالة))
        أرجع صحيح
    }

    دالة اشترك(الموضوع) {
        هذا.اشتراكات.أضف(الموضوع)
        أرجع صحيح
    }

    دالة تكرار_الاتصال() {
        أرجع هذا.متصل
    }
}

صنف بلوتوث_ble {
    دالة بناء(اسم_الجهاز) {
        هذا.اسم = اسم_الجهاز
        هذا.يعمل = خطأ
        هذا.قيم_الخصائص = {}
    }

    دالة ابدأ(معرف_الخدمة) {
        هذا.خدمة = معرف_الخدمة
        هذا.يعمل = صحيح
        تسلسلي_اطبع("بدأ بث BLE باسم: " + هذا.اسم)
        أرجع صحيح
    }

    دالة اضبط_خاصية(معرف_الخاصية، قيمة) {
        دع ق = هذا.قيم_الخصائص
        ق[معرف_الخاصية] = قيمة
        أرجع صحيح
    }

    دالة اقرأ_خاصية(معرف_الخاصية) {
        إذا (معرف_الخاصية في هذا.قيم_الخصائص) {
            أرجع هذا.قيم_الخصائص[معرف_الخاصية]
        }
        أرجع ""
    }
}

صنف حساس_لمس_esp {
    دالة بناء(منفذ_لمس) {
        هذا.منفذ = منفذ_لمس
        هذا.عتبة = 40
    }

    دالة اقرأ_قيمة() {
        أرجع 28
    }

    دالة هل_ملموس() {
        أرجع هذا.اقرأ_قيمة() < هذا.عتبة
    }
}

صنف نوم_عميق_esp {
    دالة بناء() {}

    دالة نوم_بالمؤقت(ثواني) {
        تسلسلي_اطبع("الدخول في وضع النوم العميق لمدة " + نص(ثواني) + " ثانية...")
        أرجع صحيح
    }

    دالة نوم_باللمس(منفذ_لمس، عتبة = 40) {
        تسلسلي_اطبع("الدخول في وضع النوم العميق والاستيقاظ باللمس...")
        أرجع صحيح
    }
}

صنف ذاكرة_فلاش_spiffs {
    دالة بناء() {
        هذا.ملفات = {}
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة اكتب_ملف(مسار، محتوى) {
        دع ف = هذا.ملفات
        ف[مسار] = محتوى
        أرجع صحيح
    }

    دالة اقرأ_ملف(مسار) {
        إذا (مسار في هذا.ملفات) {
            أرجع هذا.ملفات[مسار]
        }
        أرجع ""
    }

    دالة هل_موجود(مسار) {
        أرجع مسار في هذا.ملفات
    }
}

صنف محول_تماثلي_esp {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.دقة = 12
    }

    دالة اقرأ_رقمي_12bit() {
        أرجع 2048
    }

    دالة اقرأ_جهد_ملي_فولت() {
        دع قراءة = هذا.اقرأ_رقمي_12bit()
        أرجع (قراءة * 3300) / 4095
    }
}

صنف محول_رقمي_esp {
    دالة بناء(منفذ_dac) {
        هذا.منفذ = منفذ_dac
    }

    دالة اكتب_قيمة(قيمة_8bit) {
        أرجع قيمة_8bit
    }

    دالة اكتب_جهد(فولت) {
        دع قيمة = (فولت / 3.3) * 255
        أرجع هذا.اكتب_قيمة(قيمة)
    }
}

صنف تحديث_هوائي_ota {
    دالة بناء(اسم_المضيف = "Lisan-ESP32"، كلمة_السر = "") {
        هذا.مضيف = اسم_المضيف
        هذا.كلمة_سر = كلمة_السر
        هذا.مفعل = خطأ
    }

    دالة ابدأ() {
        هذا.مفعل = صحيح
        تسلسلي_اطبع("تم تفعيل خدمة التحديث الهوائي OTA باسم: " + هذا.مضيف)
        أرجع صحيح
    }

    دالة تعامل() {
        أرجع هذا.مفعل
    }
}
"#;

/// المكتبة القياسية المدمجة لمتحكمات STM32 (ARM Cortex-M: Blue Pill / Black Pill / Nucleo)
pub const STDLIB_STM32: &str = r#"
# مكتبة متحكمات STM32 القياسية في لغة لسان (Hardware Timers / CAN / 12-bit DAC / RTC)

صنف مؤقت_عتادي_stm32 {
    دالة بناء(رقم_المؤقت) {
        هذا.رقم = رقم_المؤقت
        هذا.تردد_هرتز = 1000
        هذا.نسبة_النبضة = 50
    }

    دالة اضبط_تردد(هرتز) {
        هذا.تردد_هرتز = هرتز
        أرجع هرتز
    }

    دالة اضبط_pwm(منفذ، نسبة_مئوية) {
        هذا.نسبة_النبضة = نسبة_مئوية
        اكتب_تماثلي(منفذ، (نسبة_مئوية * 255) / 100)
        أرجع نسبة_مئوية
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة أوقف() {
        أرجع صحيح
    }
}

صنف محول_تماثلي_stm32 {
    دالة بناء(منفذ) {
        هذا.منفذ = منفذ
        هذا.دقة = 12
    }

    دالة اقرأ_قيمة_12bit() {
        أرجع 2048
    }

    دالة اقرأ_جهد() {
        دع خام = هذا.اقرأ_قيمة_12bit()
        أرجع (خام * 3.3) / 4095.0
    }

    دالة اقرأ_حرارة_المعالج_الداخلية() {
        أرجع 32.5
    }
}

صنف محول_رقمي_stm32 {
    دالة بناء(قناة_dac) {
        هذا.قناة = قناة_dac
    }

    دالة اكتب_قيمة_12bit(قيمة) {
        أرجع قيمة
    }

    دالة اكتب_جهد(فولت) {
        دع خام = (فولت / 3.3) * 4095.0
        أرجع هذا.اكتب_قيمة_12bit(خام)
    }
}

صنف ناقل_can_stm32 {
    دالة بناء(سرعة_كبس = 500) {
        هذا.سرعة = سرعة_كبس
        هذا.متصل = صحيح
    }

    دالة ابدأ() {
        أرجع صحيح
    }

    دالة أرسل_إطار(معرف_الرسالة، بيانات) {
        تسلسلي_اطبع("[CAN TX] ID: " + نص(معرف_الرسالة) + " Data: " + نص(بيانات))
        أرجع صحيح
    }

    دالة استقبل_إطار() {
        أرجع {"معرف": 256, "بيانات": [1, 2, 3, 4], "طول": 4}
    }
}

صنف ساعة_الوقت_الحقيقي_rtc {
    دالة بناء() {
        هذا.ساعة = 12
        هذا.دقيقة = 0
        هذا.ثانية = 0
        هذا.يوم = 15
        هذا.شهر = 11
        هذا.سنة = 2026
    }

    دالة ضبط_الوقت(ساعة، دقيقة، ثانية) {
        هذا.ساعة = ساعة
        هذا.دقيقة = دقيقة
        هذا.ثانية = ثانية
        أرجع صحيح
    }

    دالة ضبط_التاريخ(يوم، شهر، سنة) {
        هذا.يوم = يوم
        هذا.شهر = شهر
        هذا.سنة = سنة
        أرجع صحيح
    }

    دالة اقرأ_وقت_نصي() {
        أرجع نص(هذا.ساعة) + ":" + نص(هذا.دقيقة) + ":" + نص(هذا.ثانية)
    }

    دالة اقرأ_تاريخ_نصي() {
        أرجع نص(هذا.سنة) + "-" + نص(هذا.شهر) + "-" + نص(هذا.يوم)
    }
}

صنف وضع_توفير_طاقة_stm32 {
    دالة بناء() {}

    دالة وضع_سكون() {
        تسلسلي_اطبع("STM32 دخل في وضع السكون (Sleep Mode)")
        أرجع صحيح
    }

    دالة وضع_إيقاف() {
        تسلسلي_اطبع("STM32 دخل في وضع الإيقاف (Stop Mode)")
        أرجع صحيح
    }

    دالة وضع_استعداد() {
        تسلسلي_اطبع("STM32 دخل في وضع الاستعداد (Standby Mode)")
        أرجع صحيح
    }
}

صنف منفذ_تسلسلي_stm32 {
    دالة بناء(رقم_المنفذ، سرعة = 115200) {
        هذا.رقم = رقم_المنفذ
        هذا.سرعة = سرعة
    }

    دالة ابدأ() {
        تسلسلي_اطبع("بدأ تشغيل USART" + نص(هذا.رقم) + " بسرعة " + نص(هذا.سرعة))
        أرجع صحيح
    }

    دالة اكتب_نص(نص_الرسالة) {
        تسلسلي_اطبع("[USART" + نص(هذا.رقم) + "] " + نص(نص_الرسالة))
        أرجع صحيح
    }

    دالة أرسل(نص_الرسالة) {
        أرجع هذا.اكتب_نص(نص_الرسالة)
    }

    دالة اقرأ() {
        أرجع ""
    }
}
"#;

/// المكتبة المدمجة الشاملة لجميع المتحكمات الدقيقة
pub const STDLIB_EMBEDDED: &str = r#"
# مكتبة المتحكمات الشاملة (Arduino + ESP32 + STM32)
# تتضمن كافة الأصناف والعتاد المدعوم في منصة لسان المدمجة
"#;

/// نوع الشريحة أو اللوحة المستهدفة بالترجمة
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MicroTarget {
    Universal,
    Arduino,
    Esp32,
    Stm32,
}

/// تحويل شيفرة لسان إلى كود C++ قياسي للأردوينو والمتحكمات الدقيقة (.ino)
pub fn transpile_arduino(source: &str) -> Result<String, String> {
    transpile_target(source, MicroTarget::Universal)
}

/// تحويل شيفرة لسان إلى كود C++ مخصص لمتحكمات ESP32 / ESP8266
pub fn transpile_esp32(source: &str) -> Result<String, String> {
    transpile_target(source, MicroTarget::Esp32)
}

/// تحويل شيفرة لسان إلى كود C++ مخصص لمتحكمات STM32
pub fn transpile_stm32(source: &str) -> Result<String, String> {
    transpile_target(source, MicroTarget::Stm32)
}

/// تحويل شيفرة لسان إلى كود C++ بحسب اللوحة المستهدفة
pub fn transpile_target(source: &str, target: MicroTarget) -> Result<String, String> {
    let tokens = crate::lexer::tokenize(source)?;
    let statements = crate::parser::parse(tokens)?;
    transpile_statements_to_ino(&statements, source, target)
}

/// بناء وحفظ ملف أردوينو (.ino) من ملف لسان (.lsn)
pub fn build_arduino_file(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    build_target_file(input, output, MicroTarget::Universal)
}

/// بناء وحفظ ملف C++ للمتحكم المستهدف من ملف لسان (.lsn)
pub fn build_target_file(input: &Path, output: Option<&Path>, target: MicroTarget) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| format!("تعذرت قراءة ملف لسان '{}': {}", input.display(), e))?;
    let ino_code = transpile_target(&source, target)?;

    let out_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("sketch");
            PathBuf::from(format!("{}.ino", stem))
        }
    };

    std::fs::write(&out_path, ino_code)
        .map_err(|e| format!("تعذر حفظ ملف المتحكم '{}': {}", out_path.display(), e))?;
    Ok(out_path)
}

fn transpile_statements_to_ino(statements: &[Stmt], raw_source: &str, target: MicroTarget) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("// ==================================================================\n");
    out.push_str("// تم توليد كود المتحكمات تلقائياً بواسطة لغة لسان (Lisan)\n");
    match target {
        MicroTarget::Arduino => {
            out.push_str("// اللوحة المستهدفة: لوحات أردوينو القياسية (Arduino Uno / Nano / Mega)\n");
            out.push_str("#define LISAN_TARGET_ARDUINO\n");
        }
        MicroTarget::Esp32 => {
            out.push_str("// اللوحة المستهدفة: متحكمات ESP32 / ESP8266 (WiFi / BLE / FreeRTOS)\n");
            out.push_str("#define LISAN_TARGET_ESP32\n");
        }
        MicroTarget::Stm32 => {
            out.push_str("// اللوحة المستهدفة: متحكمات STM32 (ARM Cortex-M: F103 / F401 / Nucleo)\n");
            out.push_str("#define LISAN_TARGET_STM32\n");
        }
        MicroTarget::Universal => {
            out.push_str("// اللوحات المدعومة: Arduino Uno / Nano / Mega / ESP32 / ESP8266 / STM32\n");
        }
    }
    out.push_str("// ==================================================================\n\n");
    out.push_str("#include <Arduino.h>\n");

    // مكتبات الأردوينو القياسية
    if raw_source.contains("محرك_سيرفو") {
        out.push_str("#include <Servo.h>\n");
    }
    if raw_source.contains("محرك_خطوي") {
        out.push_str("#include <Stepper.h>\n");
    }
    if raw_source.contains("شاشة_كريستال") {
        out.push_str("#include <Wire.h>\n#include <LiquidCrystal_I2C.h>\n");
    }
    if raw_source.contains("شاشة_oled") {
        out.push_str("#include <Wire.h>\n#include <Adafruit_GFX.h>\n#include <Adafruit_SSD1306.h>\n");
    }
    if raw_source.contains("محدد_مواقع_gps") {
        out.push_str("#include <TinyGPSPlus.h>\n");
    }
    if raw_source.contains("حساس_حرارة") {
        out.push_str("#include <DHT.h>\n");
    }
    if raw_source.contains("شريط_إضاءة") {
        out.push_str("#include <Adafruit_NeoPixel.h>\n");
    }
    if raw_source.contains("ذاكرة_eeprom") {
        out.push_str("#include <EEPROM.h>\n");
    }
    if raw_source.contains("ناقل_i2c") {
        out.push_str("#include <Wire.h>\n");
    }
    if raw_source.contains("ناقل_spi") {
        out.push_str("#include <SPI.h>\n");
    }

    // مكتبات ESP32 و ESP8266
    if raw_source.contains("واي_فاي") || raw_source.contains("خادم_ويب") || matches!(target, MicroTarget::Esp32) {
        out.push_str("#if defined(ESP32)\n#include <WiFi.h>\n#include <WebServer.h>\n#elif defined(ESP8266)\n#include <ESP8266WiFi.h>\n#include <ESP8266WebServer.h>\n#endif\n");
    }
    if raw_source.contains("عميل_mqtt") {
        out.push_str("#include <PubSubClient.h>\n");
    }
    if raw_source.contains("بلوتوث_ble") {
        out.push_str("#if defined(ESP32)\n#include <BLEDevice.h>\n#include <BLEServer.h>\n#include <BLEUtils.h>\n#include <BLE2902.h>\n#endif\n");
    }
    if raw_source.contains("ذاكرة_فلاش_spiffs") {
        out.push_str("#if defined(ESP32)\n#include <SPIFFS.h>\n#elif defined(ESP8266)\n#include <FS.h>\n#endif\n");
    }
    if raw_source.contains("تحديث_هوائي_ota") {
        out.push_str("#include <ArduinoOTA.h>\n");
    }

    // مكتبات STM32
    if raw_source.contains("مؤقت_عتادي_stm32") || matches!(target, MicroTarget::Stm32) {
        out.push_str("#if defined(ARDUINO_ARCH_STM32)\n#include <HardwareTimer.h>\n#endif\n");
    }
    if raw_source.contains("ساعة_الوقت_الحقيقي_rtc") {
        out.push_str("#if defined(ARDUINO_ARCH_STM32)\n#include <STM32RTC.h>\n#endif\n");
    }
    if raw_source.contains("ناقل_can_stm32") {
        out.push_str("#if defined(ARDUINO_ARCH_STM32)\n#include <HalCan.h>\n#endif\n");
    }
    out.push('\n');

    let mut globals = Vec::new();
    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut has_setup = false;
    let mut has_loop = false;

    for stmt in statements {
        match stmt.unwrap_spanned() {
            Stmt::Import(_) => {}
            Stmt::Function(func) => {
                if func.name == "إعداد" || func.name == "اعداد" || func.name == "setup" {
                    has_setup = true;
                }
                if func.name == "تكرار" || func.name == "حلقة" || func.name == "loop" {
                    has_loop = true;
                }
                functions.push(func);
            }
            Stmt::Class(cls) => {
                classes.push(cls);
            }
            other => {
                globals.push(other);
            }
        }
    }

    // ترجمة الأصناف إلى أصناف C++
    for cls in &classes {
        out.push_str(&emit_cpp_class(cls));
        out.push('\n');
    }

    // المتغيرات العامة
    let mut top_level_exec = Vec::new();
    for g in &globals {
        match g {
            Stmt::Let(name, expr) => {
                let c_name = sanitize_ident(name);
                let c_type = infer_cpp_type(expr);
                let c_val = emit_cpp_expr(expr);
                out.push_str(&format!("{} {} = {};\n", c_type, c_name, c_val));
            }
            other => {
                top_level_exec.push(other);
            }
        }
    }
    if !globals.is_empty() {
        out.push('\n');
    }

    // الدوال
    for func in &functions {
        out.push_str(&emit_cpp_function(func));
        out.push('\n');
    }

    // إذا لم يُعرّف المستخدم دالة إعداد() صراحة، ننشئ setup() من الجمل العليا
    if !has_setup {
        out.push_str("void setup() {\n");
        let baud = if matches!(target, MicroTarget::Esp32 | MicroTarget::Stm32) { "115200" } else { "9600" };
        out.push_str(&format!("  Serial.begin({});\n", baud));
        for stmt in &top_level_exec {
            out.push_str(&emit_cpp_stmt(stmt, 1));
        }
        out.push_str("}\n\n");
    }

    if !has_loop {
        out.push_str("void loop() {\n  // الحلقة الرئيسية للمتحكم\n}\n");
    }

    Ok(out)
}

fn emit_cpp_class(cls: &ClassDef) -> String {
    let mut s = String::new();
    let cname = sanitize_ident(&cls.name);
    if let Some(parent) = &cls.parent {
        s.push_str(&format!("class {} : public {} {{\npublic:\n", cname, sanitize_ident(parent)));
    } else {
        s.push_str(&format!("class {} {{\npublic:\n", cname));
    }
    for m in &cls.methods {
        if m.name == "بناء" || m.name == "تهيئة" {
            let params: Vec<String> = m.params.iter().map(|p| format!("auto {}", sanitize_ident(p))).collect();
            s.push_str(&format!("  {}({}) {{\n", cname, params.join(", ")));
            for stmt in &m.body {
                s.push_str(&emit_cpp_stmt(stmt, 2));
            }
            s.push_str("  }\n");
        } else {
            let ret_type = if body_has_return(&m.body) { "auto" } else { "void" };
            let params: Vec<String> = m.params.iter().map(|p| format!("auto {}", sanitize_ident(p))).collect();
            s.push_str(&format!("  {} {}({}) {{\n", ret_type, sanitize_ident(&m.name), params.join(", ")));
            for stmt in &m.body {
                s.push_str(&emit_cpp_stmt(stmt, 2));
            }
            s.push_str("  }\n");
        }
    }
    s.push_str("};\n");
    s
}

fn emit_cpp_function(func: &Function) -> String {
    let mut s = String::new();
    let (ret_type, fn_name) = match func.name.as_str() {
        "إعداد" | "اعداد" | "setup" => ("void", "setup".to_string()),
        "تكرار" | "حلقة" | "loop" => ("void", "loop".to_string()),
        other => {
            let rt = if body_has_return(&func.body) { "double" } else { "void" };
            (rt, sanitize_ident(other))
        }
    };
    let params: Vec<String> = func
        .params
        .iter()
        .map(|p| format!("double {}", sanitize_ident(p)))
        .collect();
    s.push_str(&format!("{} {}({}) {{\n", ret_type, fn_name, params.join(", ")));
    for stmt in &func.body {
        s.push_str(&emit_cpp_stmt(stmt, 1));
    }
    s.push_str("}\n");
    s
}

fn body_has_return(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt.unwrap_spanned() {
            Stmt::Return(Some(_)) => return true,
            Stmt::If(_, yes, no) => {
                if body_has_return(yes) {
                    return true;
                }
                if let Some(n) = no {
                    if body_has_return(n) {
                        return true;
                    }
                }
            }
            Stmt::While(_, body) | Stmt::ForEach(_, _, body) | Stmt::Block(body) => {
                if body_has_return(body) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn emit_cpp_stmt(stmt: &Stmt, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match stmt {
        Stmt::Spanned(inner, _, _) => emit_cpp_stmt(inner, indent),
        Stmt::Let(name, expr) => {
            let c_type = infer_cpp_type(expr);
            format!("{}{} {} = {};\n", pad, c_type, sanitize_ident(name), emit_cpp_expr(expr))
        }
        Stmt::Print(expr) => {
            format!("{}Serial.println({});\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Expr(expr) => {
            format!("{}{};\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Return(Some(expr)) => {
            format!("{}return {};\n", pad, emit_cpp_expr(expr))
        }
        Stmt::Return(None) => format!("{}return;\n", pad),
        Stmt::Break => format!("{}break;\n", pad),
        Stmt::Continue => format!("{}continue;\n", pad),
        Stmt::Block(stmts) => {
            let mut s = format!("{}{{\n", pad);
            for inner in stmts {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::If(cond, yes, no) => {
            let mut s = format!("{}if ({}) {{\n", pad, emit_cpp_expr(cond));
            for inner in yes {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            if let Some(no_body) = no {
                s.push_str(&format!("{}}} else {{\n", pad));
                for inner in no_body {
                    s.push_str(&emit_cpp_stmt(inner, indent + 1));
                }
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::While(cond, body) => {
            let mut s = format!("{}while ({}) {{\n", pad, emit_cpp_expr(cond));
            for inner in body {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        Stmt::ForEach(var, iter, body) => {
            let mut s = format!(
                "{}for (auto {} : {}) {{\n",
                pad,
                sanitize_ident(var),
                emit_cpp_expr(iter)
            );
            for inner in body {
                s.push_str(&emit_cpp_stmt(inner, indent + 1));
            }
            s.push_str(&format!("{}}}\n", pad));
            s
        }
        _ => String::new(),
    }
}

fn infer_cpp_type(expr: &Expr) -> &'static str {
    match expr {
        Expr::Spanned(inner, _, _) => infer_cpp_type(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 {
                "int"
            } else {
                "double"
            }
        }
        Expr::Text(_) => "String",
        Expr::Bool(_) => "bool",
        _ => "auto",
    }
}

fn emit_cpp_expr(expr: &Expr) -> String {
    match expr {
        Expr::Spanned(inner, _, _) => emit_cpp_expr(inner),
        Expr::Number(n) => {
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Text(t) => format!("String({:?})", t),
        Expr::Bool(true) => "true".into(),
        Expr::Bool(false) => "false".into(),
        Expr::Null => "0".into(),
        Expr::Variable(name) => match name.as_str() {
            "عالي" | "مرتفع" | "HIGH" => "HIGH".into(),
            "منخفض" | "LOW" => "LOW".into(),
            "مخرج" | "OUTPUT" => "OUTPUT".into(),
            "مدخل" | "INPUT" => "INPUT".into(),
            "مدخل_سحب" | "INPUT_PULLUP" => "INPUT_PULLUP".into(),
            "هذا" | "ذاتي" => "(*this)".into(),
            other => sanitize_ident(other),
        },
        Expr::Assign(name, val) => {
            format!("{} = {}", sanitize_ident(name), emit_cpp_expr(val))
        }
        Expr::Binary(left, op, right) => {
            let l = emit_cpp_expr(left);
            let r = emit_cpp_expr(right);
            if matches!(op, Op::Power) {
                return format!("pow({}, {})", l, r);
            }
            let op_str = match op {
                Op::Add => "+",
                Op::Subtract => "-",
                Op::Multiply => "*",
                Op::Divide => "/",
                Op::Modulo => "%",
                Op::Power => "^",
                Op::Equal => "==",
                Op::NotEqual => "!=",
                Op::Less => "<",
                Op::LessEqual => "<=",
                Op::Greater => ">",
                Op::GreaterEqual => ">=",
                Op::And => "&&",
                Op::Or => "||",
                Op::Not => "!",
                Op::Negate => "-",
                _ => "==",
            };
            format!("({} {} {})", l, op_str, r)
        }
        Expr::Unary(op, inner) => {
            let v = emit_cpp_expr(inner);
            match op {
                Op::Negate => format!("(-{})", v),
                Op::Not => format!("(!{})", v),
                _ => v,
            }
        }
        Expr::Call(name, args) => emit_cpp_call(name, args),
        Expr::GetProperty(target, prop) => {
            format!("{}.{}", emit_cpp_expr(target), sanitize_ident(prop))
        }
        Expr::SetProperty(target, prop, val) => {
            format!(
                "{}.{} = {}",
                emit_cpp_expr(target),
                sanitize_ident(prop),
                emit_cpp_expr(val)
            )
        }
        Expr::MethodCall(target, method, args) => {
            let c_args: Vec<String> = args.iter().map(emit_cpp_expr).collect();
            format!(
                "{}.{}({})",
                emit_cpp_expr(target),
                sanitize_ident(method),
                c_args.join(", ")
            )
        }
        Expr::Index(target, idx) => {
            format!("{}[{}]", emit_cpp_expr(target), emit_cpp_expr(idx))
        }
        Expr::AssignIndex(name, idx, val) => {
            format!(
                "{}[{}] = {}",
                sanitize_ident(name),
                emit_cpp_expr(idx),
                emit_cpp_expr(val)
            )
        }
        Expr::List(items) => {
            let vals: Vec<String> = items.iter().map(emit_cpp_expr).collect();
            format!("{{{}}}", vals.join(", "))
        }
        _ => "0".into(),
    }
}

fn emit_cpp_call(name: &str, args: &[Expr]) -> String {
    let c_args: Vec<String> = args.iter().map(emit_cpp_expr).collect();
    match name {
        "وضع_المنفذ" if args.len() == 2 => {
            let mode = match args[1].unwrap_spanned() {
                Expr::Text(s) => match s.as_str() {
                    "مخرج" | "OUTPUT" => "OUTPUT".to_string(),
                    "مدخل" | "INPUT" => "INPUT".to_string(),
                    "مدخل_سحب" | "INPUT_PULLUP" => "INPUT_PULLUP".to_string(),
                    _ => c_args[1].clone(),
                },
                _ => c_args[1].clone(),
            };
            format!("pinMode({}, {})", c_args[0], mode)
        }
        "اكتب_رقمي" if args.len() == 2 => {
            let state = match args[1].unwrap_spanned() {
                Expr::Number(n) => {
                    if *n != 0.0 {
                        "HIGH".to_string()
                    } else {
                        "LOW".to_string()
                    }
                }
                Expr::Bool(b) => {
                    if *b {
                        "HIGH".to_string()
                    } else {
                        "LOW".to_string()
                    }
                }
                Expr::Text(s) if s == "عالي" || s == "HIGH" => "HIGH".to_string(),
                Expr::Text(s) if s == "منخفض" || s == "LOW" => "LOW".to_string(),
                _ => c_args[1].clone(),
            };
            format!("digitalWrite({}, {})", c_args[0], state)
        }
        "اقرأ_رقمي" if args.len() == 1 => format!("digitalRead({})", c_args[0]),
        "اكتب_تماثلي" if args.len() == 2 => format!("analogWrite({}, {})", c_args[0], c_args[1]),
        "اقرأ_تماثلي" if args.len() == 1 => format!("analogRead({})", c_args[0]),
        "اقرأ_لمس" if args.len() == 1 => format!("touchRead({})", c_args[0]),
        "اكتب_dac" | "اكتب_محول_رقمي" if args.len() == 2 => format!("dacWrite({}, {})", c_args[0], c_args[1]),
        "دقة_التماثلي" if args.len() == 1 => format!("analogReadResolution({})", c_args[0]),
        "دقة_الكتابة_التماثلية" if args.len() == 1 => format!("analogWriteResolution({})", c_args[0]),
        "نوم_عميق" if args.len() == 1 => format!("esp_deep_sleep(({}) * 1000000ULL)", c_args[0]),
        "إعادة_تشغيل" | "اعادة_تشغيل" if args.is_empty() => {
            "ESP.restart()".into()
        }
        "انتظر_ملي" | "تأخير" | "تاخير" if args.len() == 1 => format!("delay({})", c_args[0]),
        "انتظر_ميكرو" if args.len() == 1 => format!("delayMicroseconds({})", c_args[0]),
        "وقت_التشغيل" if args.is_empty() => "millis()".into(),
        "نبضة_صوت" if args.len() >= 2 => {
            if args.len() == 3 {
                format!("tone({}, {}, {})", c_args[0], c_args[1], c_args[2])
            } else {
                format!("tone({}, {})", c_args[0], c_args[1])
            }
        }
        "أوقف_صوت" | "اوقف_صوت" if args.len() == 1 => format!("noTone({})", c_args[0]),
        "تسلسلي_ابدأ" | "تسلسلي_ابدا" if args.len() == 1 => format!("Serial.begin({})", c_args[0]),
        "تسلسلي_اطبع" if args.len() == 1 => format!("Serial.println({})", c_args[0]),
        "تسلسلي_متاح" if args.is_empty() => "Serial.available()".into(),
        "تسلسلي_اقرأ" | "تسلسلي_اقرا" if args.is_empty() => "Serial.readString()".into(),
        "خريطة_قيمة" if args.len() == 5 => format!(
            "map({}, {}, {}, {}, {})",
            c_args[0], c_args[1], c_args[2], c_args[3], c_args[4]
        ),
        "قيد_قيمة" if args.len() == 3 => {
            format!("constrain({}, {}, {})", c_args[0], c_args[1], c_args[2])
        }
        "نص" if args.len() == 1 => format!("String({})", c_args[0]),
        "مطلق" if args.len() == 1 => format!("abs({})", c_args[0]),
        "جذر" if args.len() == 1 => format!("sqrt({})", c_args[0]),
        other => format!("{}({})", sanitize_ident(other), c_args.join(", ")),
    }
}

fn sanitize_ident(name: &str) -> String {
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return name.to_string();
    }
    let mut out = String::from("lsn_");
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push_str(&format!("{:x}", c as u32));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpile_arduino_servo() {
        let code = r#"
استورد "اردوينو"
دع سيرفو = جديد محرك_سيرفو(9)
سيرفو.حرك(120)
"#;
        let ino = transpile_arduino(code).expect("يجب أن تترجم شيفرة السيرفو بنجاح");
        assert!(ino.contains("#include <Servo.h>"));
        assert!(ino.contains("void setup()"));
        assert!(ino.contains("void loop()"));
    }

    #[test]
    fn test_transpile_esp32_wifi_and_webserver() {
        let code = r#"
استورد "esp32"
دع واي_فاي = جديد واي_فاي_esp()
واي_فاي.اتصل("شبكتي"، "كلمة_المرور")
دع خادم = جديد خادم_ويب_esp(80)
خادم.عند_طلب("/", "مرحبا من ESP32 بلغة لسان")
خادم.ابدأ()
"#;
        let ino = transpile_esp32(code).expect("يجب أن تترجم شيفرة ESP32 بنجاح");
        assert!(ino.contains("LISAN_TARGET_ESP32"));
        assert!(ino.contains("#include <WiFi.h>"));
        assert!(ino.contains("#include <WebServer.h>"));
    }

    #[test]
    fn test_transpile_stm32_timer_and_can() {
        let code = r#"
استورد "stm32"
دع مؤقت = جديد مؤقت_عتادي_stm32(1)
مؤقت.اضبط_تردد(2000)
مؤقت.اضبط_pwm(5، 75)
دع كان = جديد ناقل_can_stm32(500)
كان.ابدأ()
كان.أرسل_إطار(100، [10, 20, 30])
"#;
        let ino = transpile_stm32(code).expect("يجب أن تترجم شيفرة STM32 بنجاح");
        assert!(ino.contains("LISAN_TARGET_STM32"));
        assert!(ino.contains("#include <HardwareTimer.h>"));
        assert!(ino.contains("#include <HalCan.h>"));
    }
}

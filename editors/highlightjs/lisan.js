/*
 Language: Lisan (لسان)
 Description: Arabic programming language. Keywords match src/lexer.rs of the lisan repo.
 Website: https://github.com/gitmofficiel-dot/lisan
 Category: scripting
 Usage (browser):  hljs.registerLanguage('lisan', window.hljsLisan)  then  <code class="language-lisan">
 Usage (node):     hljs.registerLanguage('lisan', require('./lisan.js'))
*/
function lisan(hljs) {
  const KEYWORDS = {
    $pattern: /[\p{L}\p{M}\p{N}_ـ]+/u,
    keyword: [
      'دع','ليكن','متغير','اطبع','إذا','اذا','وإلا','والا','طالما','بينما','لكل','في','دالة',
      'استورد','أرجع','ارجع','توقف','تابع','و','أو','او','ليس','حاول','التقط','أمسك','امسك',
      'أخيرًا','أخيرا','اخيرا','ارمِ','ارم','ألقِ','الق','اختر','حالة','افتراضي',
      'صنف','فئة','هيكل','يرث','يمتد','جديد','مع','كـ','باسم','كاسم','من'
    ],
    literal: ['صحيح','خطأ','خطا','لاشيء'],
    built_in: [
      'اطبع','طول','نوع','نص','عدد','عدد_صحيح','الآن','مدى','أضف','احذف','عكس','فرز','مجموع','متوسط','ابحث',
      'مفاتيح','قيم','يحتوي','قسّم','ادمج','استبدل','نظّف','اقتطع','يبدأ_ب','ينتهي_ب','كرر',
      'أحرف_كبيرة','أحرف_صغيرة','مطلق','جذر','قرّب','أدنى','أعلى','أرضية','سقف','قوة',
      'اقرأ_ملف','اكتب_ملف','ألحق_ملف','ملف_موجود','ادخل','تحقق','تحقق_يساوي'
    ],
    variable: ['هذا']
  };
  const STRING = {
    scope: 'string',
    begin: '"', end: '"',
    contains: [{ scope: 'char.escape', match: /\\./ }]
  };
  const NUMBER = {
    scope: 'number',
    match: /\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b/
  };
  const COMMENT = hljs.COMMENT('#', '$');
  const COMMENT2 = hljs.COMMENT('//', '$');
  return {
    name: 'Lisan',
    aliases: ['lsn', 'lisan', 'لسان'],
    unicodeRegex: true,
    keywords: KEYWORDS,
    contains: [
      COMMENT, COMMENT2, STRING, NUMBER,
      { scope: 'title.function', match: /(?<=دالة\s+)[\p{L}_][\p{L}\p{N}_]*/u },
      { scope: 'title.class', match: /(?<=(?:صنف|فئة|هيكل)\s+)[\p{L}_][\p{L}\p{N}_]*/u }
    ]
  };
}
if (typeof module !== 'undefined') module.exports = lisan;
if (typeof window !== 'undefined') window.hljsLisan = lisan;

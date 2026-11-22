# تلوين لسان لـ highlight.js

ملف `lisan.js` يضيف لغة لسان إلى [highlight.js](https://highlightjs.org/) (مكتبة التلوين المستعملة في آلاف المواقع والمدونات). الكلمات المفتاحية مطابقة لـ `src/lexer.rs`.

```html
<script src="highlight.min.js"></script>
<script src="lisan.js"></script>
<script>hljs.registerLanguage('lisan', window.hljsLisan); hljs.highlightAll();</script>
<pre><code class="language-lisan">دع س = 5
اطبع(س * 2)</code></pre>
```

مع Node: `hljs.registerLanguage('lisan', require('./lisan.js'))`.
الاختبار: `node test.mjs` (يحتاج `npm i highlight.js`)، ويشغّل التلوين على ملفات `examples/` ويتأكد من التقاط الكلمات المفتاحية والنصوص والتعليقات وأسماء الدوال.

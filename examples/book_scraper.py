import asyncio
from nexscrape import Spider, Item, Request, Config

class BooksSpider(Spider):
    """
    مكتبة NexScrape - مثال عملي قوي لسحب البيانات.
    يقوم هذا المثال باستخراج الكتب وتفاصيلها من موقع books.toscrape.com
    مع دعم التصفح عبر الصفحات المتعددة (Pagination).
    """
    
    name = "books_scraper"
    start_urls = ["https://books.toscrape.com"]
    
    # تكوين المحرك لأقصى أداء مع مراعاة القيود
    config = Config(
        concurrency=10,        # استخراج 10 صفحات في نفس الوقت
        delay=(500, 1500),     # محاكاة السلوك البشري بتاخير عشوائي بين محاولات السحب
        max_retries=3          # إعادة المحاولة في حال فشل أي طلب
    )

    async def parse(self, response):
        print(f"📖 تم الوصول إلى: {response.url}")
        
        # استخراج جميع بطاقات الكتب المتواجدة في الصفحة الحالية
        book_cards = response.css("article.product_pod")
        
        for card in book_cards:
            # استخراج تفاصيل الكتاب
            title = card.css("h3 a::attr(title)").get()
            price = card.css("p.price_color::text").get()
            in_stock = "In stock" in card.css("p.instock.availability::text").get_all()
            
            # تنظيف السعر (إزالة رمز £)
            clean_price = float(price.replace('£', '')) if price else 0.0

            # إضافة العنصر لقائمة التصدير
            yield Item(
                url=response.url,
                title=title,
                price=clean_price,
                in_stock=in_stock
            )
            
        # محرك NexScrape يدعم جلب الصفحة التالية بسهولة!
        # العثور على رابط الصفحة التالية
        next_button = response.css("li.next a::attr(href)").get()
        if next_button:
            # بناء الرابط المطلق للصفحة القادمة بشكل صحيح
            if "catalogue/" in next_button:
                next_page_url = f"https://books.toscrape.com/{next_button}"
            else:
                next_page_url = f"https://books.toscrape.com/catalogue/{next_button}"
                
            yield Request(next_page_url)


if __name__ == "__main__":
    # تشغيل الاسبايدر وتصدير النتائج إلى ملف json تلقائياً
    asyncio.run(BooksSpider.start(output="books_results.json"))

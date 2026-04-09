"""
NexScrape Python Example — Quick Scraping

This example demonstrates the Python API for NexScrape.
"""
import nexscrape as nex


class QuotesSpider(nex.Spider):
    """Spider that scrapes quotes from quotes.toscrape.com"""

    name = "quotes"
    start_urls = ["https://quotes.toscrape.com"]

    config = nex.Config(
        concurrency=8,
        delay=(0.5, 2.0),
        timeout=30,
    )

    async def parse(self, response):
        """Extract quotes from the page."""
        for quote in response.css(".quote"):
            yield nex.Item(
                text=quote.css(".text::text").get(),
                author=quote.css(".author::text").get(),
                tags=quote.css(".tag::text").getall(),
            )

        # Follow pagination
        next_page = response.css("li.next a::attr(href)").get()
        if next_page:
            yield nex.Request(next_page, callback=self.parse)


class ProductSpider(nex.Spider):
    """Spider that scrapes product data."""

    name = "products"
    start_urls = ["https://shop.example.com/products"]

    config = nex.Config(
        concurrency=16,
        delay=(1.0, 3.0),
    )

    async def parse(self, response):
        for product in response.css(".product-card"):
            yield nex.Item(
                name=product.css("h2.title::text").get(),
                price=product.css(".price::text").get(),
                url=product.css("a::attr(href)").get_abs(response.url),
                image=product.css("img::attr(src)").get(),
            )


if __name__ == "__main__":
    # Run the quotes spider
    print("🕷️  Running QuotesSpider...")
    nex.run(QuotesSpider, output="quotes.json")
    print("✅ Done! Check quotes.json")

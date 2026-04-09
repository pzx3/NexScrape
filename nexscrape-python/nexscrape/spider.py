"""Spider base class for structured web scraping."""

from __future__ import annotations
import asyncio
import logging
from typing import AsyncGenerator, Any, Optional, List
from .request import Request
from .response import Response
from .item import Item
from .config import Config


class Spider:
    """Base class for all NexScrape spiders.

    Subclass this and override `parse()` to define your scraping logic.

    Example:
        class MySpider(Spider):
            name = "my_spider"
            start_urls = ["https://example.com"]

            async def parse(self, response):
                yield Item(
                    title=response.css("h1::text").get(),
                    url=response.url,
                )
    """

    name: str = "spider"
    start_urls: List[str] = []
    config: Config = Config()

    def __init__(self, **kwargs):
        self.log = logging.getLogger(f"nexscrape.{self.name}")
        self._items: List[dict] = []
        self._request_count = 0
        self._error_count = 0

        # Override config with kwargs
        for key, value in kwargs.items():
            if hasattr(self.config, key):
                setattr(self.config, key, value)

    async def start_requests(self) -> AsyncGenerator[Request, None]:
        """Generate initial requests. Override for custom logic."""
        for url in self.start_urls:
            yield Request(url, callback=self.parse)

    async def parse(self, response: Response) -> AsyncGenerator:
        """Parse a response. Override this in your spider."""
        raise NotImplementedError(
            f"Spider '{self.name}' must implement the parse() method"
        )

    async def on_error(self, request: Request, error: Exception):
        """Called when a request fails. Override for custom error handling."""
        self._error_count += 1
        self.log.error(f"❌ Error on {request.url}: {error}")

    async def on_start(self):
        """Called before the spider starts crawling."""
        self.log.info(f"🕷️  Spider '{self.name}' starting...")

    async def on_finish(self):
        """Called after the spider finishes crawling."""
        self.log.info(
            f"✅ Spider '{self.name}' finished. "
            f"Items: {len(self._items)} | "
            f"Requests: {self._request_count} | "
            f"Errors: {self._error_count}"
        )

    async def _run(self):
        """Internal run loop."""
        await self.on_start()

        queue = asyncio.Queue()

        # Enqueue start requests
        async for request in self.start_requests():
            await queue.put(request)

        # Process queue with workers
        workers = [
            asyncio.create_task(self._worker(queue))
            for _ in range(self.config.concurrency)
        ]

        await queue.join()

        for worker in workers:
            worker.cancel()

        await self.on_finish()

    async def _worker(self, queue: asyncio.Queue):
        """Worker coroutine that processes requests from the queue."""
        while True:
            request = await queue.get()
            try:
                self._request_count += 1

                # In full implementation, this would use the Rust HTTP engine
                # For now, use a placeholder
                self.log.debug(f"Processing: {request.url}")

                # Simulated response (will be replaced by Rust engine)
                response = Response._placeholder(request.url)

                if request.callback:
                    async for result in request.callback(response):
                        if isinstance(result, Item):
                            self._items.append(result.to_dict())
                        elif isinstance(result, Request):
                            await queue.put(result)

            except Exception as e:
                await self.on_error(request, e)
            finally:
                queue.task_done()

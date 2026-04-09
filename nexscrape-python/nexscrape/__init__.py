# NexScrape Python Package

"""
NexScrape — Next-generation web scraping library.

Built with a Rust core for blazing-fast performance,
with a clean Python API for ease of use.

Quick Start:
    >>> import nexscrape as nex
    >>> response = nex.fetch("https://example.com")
    >>> print(response.title)
"""

__version__ = "0.1.0"
__author__ = "NexScrape Contributors"

from .spider import Spider
from .request import Request, FormRequest
from .response import Response
from .item import Item
from .config import Config, StealthConfig, ProxyPool, RateLimit

__all__ = [
    "Spider",
    "Request",
    "FormRequest",
    "Response",
    "Item",
    "Config",
    "StealthConfig",
    "ProxyPool",
    "RateLimit",
    "fetch",
    "run",
]


def fetch(url: str, **kwargs) -> "Response":
    """Quick one-shot fetch of a single URL.

    Args:
        url: The URL to fetch.
        **kwargs: Additional request options.

    Returns:
        Response object with parsed content.
    """
    import asyncio
    from .request import Request
    req = Request(url, **kwargs)
    # In full implementation, this would use the Rust engine
    raise NotImplementedError("Rust bindings not yet compiled. Use `maturin develop` to build.")


# def chain(url: str) -> "ChainBuilder":
#     """Start a fluent chain for quick scraping.
# 
#     Args:
#         url: Starting URL.
# 
#     Returns:
#         ChainBuilder for method chaining.
#     """
#     # from .chain import ChainBuilder
#     # return ChainBuilder(url)


def run(spider_class, output: str = None, **kwargs):
    """Run a spider class.

    Args:
        spider_class: Spider class to instantiate and run.
        output: Optional output file path.
        **kwargs: Additional spider configuration.
    """
    import asyncio
    spider = spider_class(**kwargs)
    asyncio.run(spider._run())

    if output and spider._items:
        import json
        with open(output, 'w', encoding='utf-8') as f:
            json.dump(spider._items, f, ensure_ascii=False, indent=2)

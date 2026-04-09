"""Configuration classes for NexScrape."""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import List, Optional, Tuple


@dataclass
class Config:
    """Main spider configuration.

    Attributes:
        concurrency: Number of concurrent workers.
        delay: Random delay range (min, max) in seconds between requests.
        timeout: Request timeout in seconds.
        retry: Maximum number of retries per request.
        max_depth: Maximum crawl depth (None = unlimited).
        respect_robots_txt: Whether to respect robots.txt.
        js_render: Whether to render JavaScript (requires browser).
        user_agent: Default User-Agent string.
    """

    concurrency: int = 16
    delay: Tuple[float, float] = (0.5, 2.0)
    timeout: int = 30
    retry: int = 3
    max_depth: Optional[int] = None
    respect_robots_txt: bool = True
    js_render: bool = False
    user_agent: str = "NexScrape/0.1.0 (+https://github.com/nexscrape/nexscrape)"


@dataclass
class StealthConfig:
    """Anti-detection configuration.

    Attributes:
        fingerprint_rotation: Enable browser fingerprint rotation.
        rotation_interval: Rotate fingerprint every N requests.
        human_simulation: Enable human behavior simulation.
        auto_captcha: Auto-solve CAPTCHAs.
        captcha_provider: CAPTCHA solving provider name.
        captcha_api_key: API key for CAPTCHA solver.
    """

    fingerprint_rotation: bool = True
    rotation_interval: int = 10
    human_simulation: bool = False
    auto_captcha: bool = False
    captcha_provider: Optional[str] = None
    captcha_api_key: Optional[str] = None


@dataclass
class ProxyPool:
    """Proxy pool configuration.

    Attributes:
        proxies: List of proxy URLs.
        rotation: Rotation strategy ('round_robin', 'random', 'sticky_session').
        health_check: Enable proxy health checking.
    """

    proxies: List[str] = field(default_factory=list)
    rotation: str = "round_robin"
    health_check: bool = True

    def add(self, proxy_url: str) -> "ProxyPool":
        """Add a proxy URL."""
        self.proxies.append(proxy_url)
        return self


@dataclass
class RateLimit:
    """Rate limiting configuration.

    Attributes:
        requests_per_second: Maximum requests per second.
        burst: Maximum burst size.
        per_domain: Apply limits per domain.
        backoff_on_429: Auto backoff on HTTP 429.
    """

    requests_per_second: float = 2.0
    burst: int = 10
    per_domain: bool = True
    backoff_on_429: bool = True

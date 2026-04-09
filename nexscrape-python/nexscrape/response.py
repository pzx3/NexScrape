"""Response wrapper with data extraction utilities."""

from __future__ import annotations
from typing import Any, Dict, List, Optional
import re


class Selector:
    """CSS/XPath selector result with fluent API.

    Wraps selection results and provides convenient extraction methods.
    """

    def __init__(self, results: List[str]):
        self._results = results

    def get(self, default: Optional[str] = None) -> Optional[str]:
        """Get the first result."""
        return self._results[0] if self._results else default

    def getall(self) -> List[str]:
        """Get all results."""
        return self._results

    def get_abs(self, base_url: str = "") -> Optional[str]:
        """Get the first result as an absolute URL."""
        value = self.get()
        if value and base_url:
            from urllib.parse import urljoin
            return urljoin(base_url, value)
        return value

    def __bool__(self) -> bool:
        return len(self._results) > 0

    def __len__(self) -> int:
        return len(self._results)

    def __repr__(self) -> str:
        return f"Selector({self._results})"


class Response:
    """HTTP response with parsing utilities.

    Provides methods for extracting data using CSS selectors,
    XPath, regex, and JSON parsing.
    """

    def __init__(
        self,
        url: str,
        status: int,
        headers: Dict[str, str],
        body: bytes,
    ):
        self.url = url
        self.status = status
        self.headers = headers
        self._body = body
        self._text: Optional[str] = None

    @property
    def text(self) -> str:
        """Response body as text."""
        if self._text is None:
            self._text = self._body.decode("utf-8", errors="replace")
        return self._text

    @property
    def is_success(self) -> bool:
        """True if status code is 2xx."""
        return 200 <= self.status < 300

    def css(self, selector: str) -> Selector:
        """Select elements using CSS selector.

        Supports pseudo-selectors:
        - `::text` — extract text content
        - `::attr(name)` — extract attribute value

        Note: Full CSS selection requires the Rust engine.
        This is a placeholder using regex-based extraction.
        """
        # Placeholder implementation using regex
        # Full implementation will use the Rust HTML parser
        results = []

        if "::text" in selector:
            tag = selector.split("::text")[0].strip()
            tag_name = tag.split(".")[-1] if "." in tag else tag
            pattern = rf"<{tag_name}[^>]*>(.*?)</{tag_name}>"
            matches = re.findall(pattern, self.text, re.DOTALL)
            results = [m.strip() for m in matches]
        elif "::attr(" in selector:
            parts = selector.split("::attr(")
            attr = parts[1].rstrip(")")
            tag = parts[0].strip()
            pattern = rf'{attr}=["\']([^"\']*)["\']'
            results = re.findall(pattern, self.text)
        else:
            # Basic tag content extraction
            pattern = rf"<{selector}[^>]*>(.*?)</{selector}>"
            results = re.findall(pattern, self.text, re.DOTALL)

        return Selector(results)

    def json(self) -> Any:
        """Parse response body as JSON."""
        import json
        return json.loads(self._body)

    def re(self, pattern: str) -> List[str]:
        """Extract data using regex."""
        return re.findall(pattern, self.text)

    def re_first(self, pattern: str, default: Optional[str] = None) -> Optional[str]:
        """Extract first regex match."""
        matches = self.re(pattern)
        return matches[0] if matches else default

    @classmethod
    def _placeholder(cls, url: str) -> "Response":
        """Create a placeholder response (for development)."""
        return cls(
            url=url,
            status=200,
            headers={},
            body=b"<html><body>Placeholder</body></html>",
        )

    def __repr__(self) -> str:
        return f"Response({self.status} {self.url})"

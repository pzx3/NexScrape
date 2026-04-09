"""Request types for NexScrape."""

from __future__ import annotations
from typing import Any, Callable, Dict, Optional


class Request:
    """Represents an HTTP request to be processed by the engine.

    Args:
        url: Target URL.
        method: HTTP method (GET, POST, etc.).
        headers: Custom headers dict.
        callback: Async function to call with the response.
        meta: Metadata to pass through to the callback.
        priority: Request priority (higher = first).
        dont_filter: Skip URL deduplication if True.
    """

    def __init__(
        self,
        url: str,
        method: str = "GET",
        headers: Optional[Dict[str, str]] = None,
        body: Optional[str] = None,
        callback: Optional[Callable] = None,
        meta: Optional[Dict[str, Any]] = None,
        priority: int = 0,
        dont_filter: bool = False,
    ):
        self.url = url
        self.method = method
        self.headers = headers or {}
        self.body = body
        self.callback = callback
        self.meta = meta or {}
        self.priority = priority
        self.dont_filter = dont_filter

    def __repr__(self) -> str:
        return f"Request({self.method} {self.url})"


class FormRequest(Request):
    """Request that submits form data.

    Args:
        url: Form action URL.
        form_data: Dictionary of form fields.
        **kwargs: Additional Request arguments.
    """

    def __init__(
        self,
        url: str,
        form_data: Dict[str, str],
        method: str = "POST",
        **kwargs,
    ):
        # URL-encode form data
        import urllib.parse
        body = urllib.parse.urlencode(form_data)

        headers = kwargs.pop("headers", {})
        headers.setdefault("Content-Type", "application/x-www-form-urlencoded")

        super().__init__(
            url=url,
            method=method,
            body=body,
            headers=headers,
            **kwargs,
        )
        self.form_data = form_data

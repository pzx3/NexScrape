"""Tests for NexScrape Python package."""
import pytest
from nexscrape.item import Item
from nexscrape.request import Request, FormRequest
from nexscrape.response import Response, Selector
from nexscrape.config import Config, StealthConfig, ProxyPool, RateLimit


class TestItem:
    def test_create_item(self):
        item = Item(title="Test", price=9.99)
        assert item.title == "Test"
        assert item.price == 9.99

    def test_item_to_dict(self):
        item = Item(name="Widget", url="https://example.com")
        d = item.to_dict()
        assert d["name"] == "Widget"
        assert d["url"] == "https://example.com"

    def test_item_contains(self):
        item = Item(title="Test")
        assert "title" in item
        assert "missing" not in item

    def test_item_get_default(self):
        item = Item(name="Test")
        assert item.get("name") == "Test"
        assert item.get("missing", "default") == "default"

    def test_item_repr(self):
        item = Item(x=1)
        assert "Item(" in repr(item)


class TestRequest:
    def test_get_request(self):
        req = Request("https://example.com")
        assert req.url == "https://example.com"
        assert req.method == "GET"

    def test_post_request(self):
        req = Request("https://example.com", method="POST", body="data")
        assert req.method == "POST"
        assert req.body == "data"

    def test_form_request(self):
        req = FormRequest(
            "https://example.com/login",
            form_data={"user": "admin", "pass": "secret"},
        )
        assert req.method == "POST"
        assert "user=admin" in req.body
        assert req.headers["Content-Type"] == "application/x-www-form-urlencoded"

    def test_request_repr(self):
        req = Request("https://example.com")
        assert "GET" in repr(req)
        assert "example.com" in repr(req)


class TestResponse:
    def test_response_text(self):
        resp = Response(
            url="https://example.com",
            status=200,
            headers={},
            body=b"<html>Hello</html>",
        )
        assert "Hello" in resp.text

    def test_response_is_success(self):
        resp = Response("https://example.com", 200, {}, b"")
        assert resp.is_success
        resp404 = Response("https://example.com", 404, {}, b"")
        assert not resp404.is_success

    def test_response_json(self):
        resp = Response(
            "https://api.example.com",
            200,
            {"Content-Type": "application/json"},
            b'{"key": "value"}',
        )
        data = resp.json()
        assert data["key"] == "value"

    def test_response_regex(self):
        resp = Response("https://example.com", 200, {}, b"Price: $9.99 and $19.99")
        prices = resp.re(r"\$\d+\.\d+")
        assert len(prices) == 2
        assert prices[0] == "$9.99"

    def test_response_re_first(self):
        resp = Response("https://example.com", 200, {}, b"Email: test@example.com")
        email = resp.re_first(r"[\w.]+@[\w.]+")
        assert email == "test@example.com"

    def test_placeholder(self):
        resp = Response._placeholder("https://example.com")
        assert resp.status == 200


class TestSelector:
    def test_selector_get(self):
        sel = Selector(["first", "second"])
        assert sel.get() == "first"

    def test_selector_getall(self):
        sel = Selector(["a", "b", "c"])
        assert sel.getall() == ["a", "b", "c"]

    def test_selector_empty(self):
        sel = Selector([])
        assert sel.get() is None
        assert sel.get("default") == "default"
        assert not sel

    def test_selector_len(self):
        sel = Selector(["a", "b"])
        assert len(sel) == 2


class TestConfig:
    def test_default_config(self):
        config = Config()
        assert config.concurrency == 16
        assert config.timeout == 30
        assert config.respect_robots_txt is True

    def test_custom_config(self):
        config = Config(concurrency=50, timeout=60)
        assert config.concurrency == 50
        assert config.timeout == 60


class TestStealthConfig:
    def test_default_stealth(self):
        stealth = StealthConfig()
        assert stealth.fingerprint_rotation is True
        assert stealth.auto_captcha is False


class TestProxyPool:
    def test_add_proxy(self):
        pool = ProxyPool()
        pool.add("http://proxy1:8080")
        pool.add("socks5://proxy2:1080")
        assert len(pool.proxies) == 2

    def test_chain_add(self):
        pool = ProxyPool().add("http://p1:80").add("http://p2:80")
        assert len(pool.proxies) == 2


class TestRateLimit:
    def test_default_rate(self):
        rate = RateLimit()
        assert rate.requests_per_second == 2.0
        assert rate.per_domain is True

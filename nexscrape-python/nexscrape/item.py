"""Data item container for extracted data."""

from __future__ import annotations
from typing import Any, Dict


class Item:
    """Container for extracted data fields.

    Items are yielded from spider parse methods and collected
    for export.

    Example:
        yield Item(
            title="Product Name",
            price=9.99,
            url="https://example.com/product",
        )
    """

    def __init__(self, **fields):
        self._fields: Dict[str, Any] = fields

    def __setattr__(self, name: str, value: Any):
        if name.startswith("_"):
            super().__setattr__(name, value)
        else:
            self._fields[name] = value

    def __getattr__(self, name: str) -> Any:
        if name.startswith("_"):
            raise AttributeError(name)
        try:
            return self._fields[name]
        except KeyError:
            raise AttributeError(f"Item has no field '{name}'")

    def __contains__(self, key: str) -> bool:
        return key in self._fields

    def __repr__(self) -> str:
        fields = ", ".join(f"{k}={v!r}" for k, v in self._fields.items())
        return f"Item({fields})"

    def to_dict(self) -> Dict[str, Any]:
        """Convert to a plain dictionary."""
        return dict(self._fields)

    def keys(self):
        """Get field names."""
        return self._fields.keys()

    def values(self):
        """Get field values."""
        return self._fields.values()

    def items(self):
        """Get field name-value pairs."""
        return self._fields.items()

    def get(self, key: str, default: Any = None) -> Any:
        """Get a field value with a default."""
        return self._fields.get(key, default)

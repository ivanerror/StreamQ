"""Entry point script for StreamQ application."""

import sys
import tkinter as tk
from tkinter import messagebox
try:
    import ttkbootstrap as tb
except Exception:
    tb = None

try:
    # Try to import from the new structure first
    from src.streamq.core.app import StreamQApp
except ImportError:
    # Fallback: if we can't import from src structure, try direct import
    # This allows gradual migration or running from either structure
    try:
        from streamq.core.app import StreamQApp
    except ImportError:
        messagebox.showerror(
            "Import Error", 
            "Could not import StreamQ modules. Please ensure the package is properly installed."
        )
        sys.exit(1)


def main():
    """Main entry point for StreamQ application."""
    root = tb.Window(themename="flatly") if tb else tk.Tk()
    try:
        app = StreamQApp(root)
        root.mainloop()
    except Exception as error:
        messagebox.showerror("Error", str(error))
        root.destroy()
        sys.exit(1)


if __name__ == "__main__":
    main()

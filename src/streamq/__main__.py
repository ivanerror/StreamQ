"""Entry point for StreamQ when run as a module."""

import sys
import tkinter as tk
from tkinter import messagebox

from .core.app import StreamQApp


def main():
    """Main entry point for StreamQ application."""
    root = tk.Tk()
    try:
        app = StreamQApp(root)
        root.mainloop()
    except Exception as error:
        messagebox.showerror("Error", str(error))
        root.destroy()
        sys.exit(1)


if __name__ == "__main__":
    main()
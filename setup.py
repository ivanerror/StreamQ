"""Setup script for StreamQ (fallback for older Python versions)."""

from setuptools import setup, find_packages
import os
import sys

# Ensure we're using Python 3.8+
if sys.version_info < (3, 8):
    print("StreamQ requires Python 3.8 or later.")
    sys.exit(1)

# Read the README file
def read_readme():
    readme_path = os.path.join(os.path.dirname(__file__), "README.md")
    if os.path.exists(readme_path):
        with open(readme_path, "r", encoding="utf-8") as f:
            return f.read()
    return ""

# Read requirements
def read_requirements():
    req_path = os.path.join(os.path.dirname(__file__), "requirements.txt")
    if os.path.exists(req_path):
        with open(req_path, "r", encoding="utf-8") as f:
            return [line.strip() for line in f if line.strip() and not line.startswith("#")]
    return ["yt-dlp>=2025.09.26"]

setup(
    name="streamq",
    version="1.0.0",
    author="ivanerror",
    author_email="contact@example.com",
    description="Queue and download video/audio from YouTube using yt-dlp with a clean GUI",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/ivanerror/streamq",
    project_urls={
        "Bug Reports": "https://github.com/ivanerror/streamq/issues",
        "Source": "https://github.com/ivanerror/streamq",
    },
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: End Users/Desktop",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Multimedia :: Video",
        "Topic :: Internet :: WWW/HTTP",
    ],
    python_requires=">=3.8",
    install_requires=read_requirements(),
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-cov",
            "black",
            "isort",
            "flake8",
            "mypy",
        ],
    },
    entry_points={
        "console_scripts": [
            "streamq=streamq.__main__:main",
        ],
        "gui_scripts": [
            "streamq-gui=streamq.__main__:main",
        ],
    },
    keywords="youtube download video audio yt-dlp gui tkinter",
    include_package_data=True,
    zip_safe=False,
)
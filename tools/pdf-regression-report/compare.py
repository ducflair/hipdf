#!/usr/bin/env python3
# /// script
# dependencies = [
#     "pymupdf>=1.24.0",
#     "pillow>=10.0.0",
#     "numpy>=1.26.0",
# ]
# ///

"""
hipdf PDF Regression Comparison Tool
Compares baseline vs candidate PDF test outputs, detects visual and text changes,
generates visual diff images for changed pages, and builds a static comparison report.
Also supports production-only mode for main branch baseline galleries.
"""

import argparse
import hashlib
import json
import os
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
from PIL import Image
import pymupdf as fitz


def compute_sha256(filepath: Path) -> str:
    """Compute SHA-256 hash of a file."""
    h = hashlib.sha256()
    with open(filepath, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def find_pdf_files(directory: Path) -> Dict[str, Path]:
    """Find all PDF files in a directory, keyed by their relative POSIX path."""
    if not directory.exists() or not directory.is_dir():
        return {}
    
    pdfs = {}
    for p in sorted(directory.rglob("*.pdf")):
        if p.is_file():
            rel_path = p.relative_to(directory).as_posix()
            pdfs[rel_path] = p
    return pdfs


def sanitize_filename_for_path(rel_path: str) -> str:
    """Convert relative path to a safe filename component."""
    return rel_path.replace("/", "__").replace("\\", "__").replace(".pdf", "")


def render_page_to_numpy(page: fitz.Page, dpi: int) -> Tuple[np.ndarray, int, int]:
    """Render a PDF page to a NumPy RGBA array."""
    pix = page.get_pixmap(dpi=dpi, alpha=True)
    arr = np.frombuffer(pix.samples, dtype=np.uint8).reshape((pix.height, pix.width, pix.n))
    if pix.n == 3:  # RGB -> RGBA
        alpha = np.full((pix.height, pix.width, 1), 255, dtype=np.uint8)
        arr = np.concatenate([arr, alpha], axis=2)
    return arr, pix.width, pix.height


def generate_diff_image(
    base_arr: np.ndarray,
    cand_arr: np.ndarray,
    diff_mask: np.ndarray,
    out_path: Path,
) -> None:
    """
    Generate a high-contrast visual diff image.
    Background: candidate page dimmed / grayscale.
    Differences: bright magenta/red highlighting (#FF0055).
    """
    out_path.parent.mkdir(parents=True, exist_ok=True)
    height, width = cand_arr.shape[:2]

    # Convert candidate to grayscale for subtle background
    rgb = cand_arr[:, :, :3].astype(np.float32)
    gray = (0.299 * rgb[:, :, 0] + 0.587 * rgb[:, :, 1] + 0.114 * rgb[:, :, 2]).astype(np.uint8)
    
    # Dim the grayscale background
    dimmed_gray = (gray * 0.45 + 140 * 0.55).astype(np.uint8)
    diff_vis = np.stack([dimmed_gray, dimmed_gray, dimmed_gray, np.full((height, width), 255, dtype=np.uint8)], axis=2)

    # Highlight changed pixels with vivid magenta-red [255, 0, 80]
    diff_vis[diff_mask] = [255, 0, 80, 255]

    img = Image.fromarray(diff_vis, mode="RGBA")
    img.save(out_path, format="PNG", optimize=True)


def compare_single_pdf(
    rel_path: str,
    base_path: Optional[Path],
    cand_path: Optional[Path],
    output_dir: Path,
    dpi: int = 144,
    pixel_threshold: int = 12,
    min_diff_pixels: int = 5,
) -> Dict[str, Any]:
    """Compare a single PDF pair (baseline vs candidate) and generate diff artifacts if needed."""
    
    # Case 1: Added in candidate
    if base_path is None and cand_path is not None:
        cand_sha = compute_sha256(cand_path)
        cand_size = cand_path.stat().st_size
        cand_doc = fitz.open(cand_path)
        pages_count = len(cand_doc)
        pages_info = []
        for i, page in enumerate(cand_doc):
            rect = page.rect
            pages_info.append({
                "page_number": i + 1,
                "status": "added",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
        cand_doc.close()

        # Copy candidate PDF
        dest_cand = output_dir / "candidate" / rel_path
        dest_cand.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(cand_path, dest_cand)

        return {
            "filename": rel_path,
            "status": "added",
            "baseline_path": None,
            "candidate_path": f"candidate/{rel_path}",
            "baseline_size": None,
            "candidate_size": cand_size,
            "baseline_sha256": None,
            "candidate_sha256": cand_sha,
            "baseline_pages": 0,
            "candidate_pages": pages_count,
            "sha256_match": False,
            "visual_match": False,
            "text_match": False,
            "pages_with_diff": 0,
            "pages": pages_info,
        }

    # Case 2: Removed in candidate
    if cand_path is None and base_path is not None:
        base_sha = compute_sha256(base_path)
        base_size = base_path.stat().st_size
        base_doc = fitz.open(base_path)
        pages_count = len(base_doc)
        pages_info = []
        for i, page in enumerate(base_doc):
            rect = page.rect
            pages_info.append({
                "page_number": i + 1,
                "status": "removed",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
        base_doc.close()

        # Copy baseline PDF
        dest_base = output_dir / "baseline" / rel_path
        dest_base.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(base_path, dest_base)

        return {
            "filename": rel_path,
            "status": "removed",
            "baseline_path": f"baseline/{rel_path}",
            "candidate_path": None,
            "baseline_size": base_size,
            "candidate_size": None,
            "baseline_sha256": base_sha,
            "candidate_sha256": None,
            "baseline_pages": pages_count,
            "candidate_pages": 0,
            "sha256_match": False,
            "visual_match": False,
            "text_match": False,
            "pages_with_diff": 0,
            "pages": pages_info,
        }

    # Case 3: Present in both
    assert base_path is not None and cand_path is not None
    base_sha = compute_sha256(base_path)
    cand_sha = compute_sha256(cand_path)
    base_size = base_path.stat().st_size
    cand_size = cand_path.stat().st_size

    # Copy files to output directory
    dest_base = output_dir / "baseline" / rel_path
    dest_cand = output_dir / "candidate" / rel_path
    dest_base.parent.mkdir(parents=True, exist_ok=True)
    dest_cand.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(base_path, dest_base)
    shutil.copy2(cand_path, dest_cand)

    # If exact SHA match, we know it is 100% identical without rendering
    if base_sha == cand_sha:
        base_doc = fitz.open(base_path)
        pages_count = len(base_doc)
        pages_info = []
        for i, page in enumerate(base_doc):
            rect = page.rect
            pages_info.append({
                "page_number": i + 1,
                "status": "unchanged",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
        base_doc.close()

        return {
            "filename": rel_path,
            "status": "unchanged",
            "baseline_path": f"baseline/{rel_path}",
            "candidate_path": f"candidate/{rel_path}",
            "baseline_size": base_size,
            "candidate_size": cand_size,
            "baseline_sha256": base_sha,
            "candidate_sha256": cand_sha,
            "baseline_pages": pages_count,
            "candidate_pages": pages_count,
            "sha256_match": True,
            "visual_match": True,
            "text_match": True,
            "pages_with_diff": 0,
            "pages": pages_info,
        }

    # SHA differs: Open documents and compare page by page
    base_doc = fitz.open(base_path)
    cand_doc = fitz.open(cand_path)
    base_pages = len(base_doc)
    cand_pages = len(cand_doc)
    max_pages = max(base_pages, cand_pages)

    pages_info = []
    has_visual_diff = False
    has_text_diff = False
    pages_with_diff_count = 0
    safe_rel_name = sanitize_filename_for_path(rel_path)

    for page_idx in range(max_pages):
        page_num = page_idx + 1

        if page_idx >= base_pages:
            cand_page = cand_doc[page_idx]
            rect = cand_page.rect
            pages_info.append({
                "page_number": page_num,
                "status": "added",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
            has_visual_diff = True
            pages_with_diff_count += 1
            continue

        if page_idx >= cand_pages:
            base_page = base_doc[page_idx]
            rect = base_page.rect
            pages_info.append({
                "page_number": page_num,
                "status": "removed",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
            has_visual_diff = True
            pages_with_diff_count += 1
            continue

        base_page = base_doc[page_idx]
        cand_page = cand_doc[page_idx]

        # Compare text
        base_text = base_page.get_text()
        cand_text = cand_page.get_text()
        text_is_different = (base_text != cand_text)
        if text_is_different:
            has_text_diff = True

        # Render page pixmaps
        base_arr, b_w, b_h = render_page_to_numpy(base_page, dpi=dpi)
        cand_arr, c_w, c_h = render_page_to_numpy(cand_page, dpi=dpi)

        # Pad arrays if rendered dimensions differ
        max_h = max(b_h, c_h)
        max_w = max(b_w, c_w)

        if base_arr.shape[:2] != (max_h, max_w):
            padded_b = np.full((max_h, max_w, 4), 255, dtype=np.uint8)
            padded_b[:b_h, :b_w] = base_arr
            base_arr = padded_b

        if cand_arr.shape[:2] != (max_h, max_w):
            padded_c = np.full((max_h, max_w, 4), 255, dtype=np.uint8)
            padded_c[:c_h, :c_w] = cand_arr
            cand_arr = padded_c

        # Compute pixel difference across RGB channels
        color_diff = np.abs(base_arr[:, :, :3].astype(np.int16) - cand_arr[:, :, :3].astype(np.int16))
        diff_mask = np.any(color_diff > pixel_threshold, axis=2)
        diff_pixels = int(np.count_nonzero(diff_mask))
        total_pixels = max_h * max_w
        diff_percent = round((diff_pixels / total_pixels) * 100.0, 4)

        diff_image_rel_path = None
        if diff_pixels >= min_diff_pixels:
            has_visual_diff = True
            pages_with_diff_count += 1
            diff_img_filename = f"{safe_rel_name}__page_{page_num}.png"
            diff_img_path = output_dir / "diffs" / diff_img_filename
            generate_diff_image(base_arr, cand_arr, diff_mask, diff_img_path)
            diff_image_rel_path = f"diffs/{diff_img_filename}"
            page_status = "changed"
        else:
            page_status = "unchanged"

        pages_info.append({
            "page_number": page_num,
            "status": page_status,
            "width": round(cand_page.rect.width, 2),
            "height": round(cand_page.rect.height, 2),
            "diff_pixels": diff_pixels,
            "diff_percent": diff_percent,
            "diff_image": diff_image_rel_path,
            "text_diff": None if not text_is_different else "Text content changed",
        })

    base_doc.close()
    cand_doc.close()

    overall_status = "unchanged"
    if base_pages != cand_pages or has_visual_diff:
        overall_status = "changed"
    elif has_text_diff:
        overall_status = "text_changed"

    return {
        "filename": rel_path,
        "status": overall_status,
        "baseline_path": f"baseline/{rel_path}",
        "candidate_path": f"candidate/{rel_path}",
        "baseline_size": base_size,
        "candidate_size": cand_size,
        "baseline_sha256": base_sha,
        "candidate_sha256": cand_sha,
        "baseline_pages": base_pages,
        "candidate_pages": cand_pages,
        "sha256_match": False,
        "visual_match": not has_visual_diff and (base_pages == cand_pages),
        "text_match": not has_text_diff,
        "pages_with_diff": pages_with_diff_count,
        "pages": pages_info,
    }


def generate_production_results(
    prod_dir: Path,
    output_dir: Path,
    meta: Dict[str, Any],
) -> Dict[str, Any]:
    """Scan and package production PDFs for the main branch showcase."""
    pdfs = find_pdf_files(prod_dir)
    files_info = []
    total_pages = 0

    dest_prod_dir = output_dir / "production"
    dest_prod_dir.mkdir(parents=True, exist_ok=True)

    for rel_path, path in pdfs.items():
        sha = compute_sha256(path)
        size = path.stat().st_size
        doc = fitz.open(path)
        pages_count = len(doc)
        total_pages += pages_count

        pages_info = []
        for i, page in enumerate(doc):
            rect = page.rect
            pages_info.append({
                "page_number": i + 1,
                "status": "unchanged",
                "width": round(rect.width, 2),
                "height": round(rect.height, 2),
                "diff_pixels": 0,
                "diff_percent": 0.0,
                "diff_image": None,
                "text_diff": None,
            })
        doc.close()

        dest_pdf = dest_prod_dir / rel_path
        dest_pdf.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, dest_pdf)

        files_info.append({
            "filename": rel_path,
            "status": "unchanged",
            "production_path": f"production/{rel_path}",
            "candidate_path": f"production/{rel_path}",
            "baseline_path": f"production/{rel_path}",
            "size": size,
            "candidate_size": size,
            "baseline_size": size,
            "sha256": sha,
            "pages": pages_count,
            "candidate_pages": pages_count,
            "baseline_pages": pages_count,
            "sha256_match": True,
            "visual_match": True,
            "text_match": True,
            "pages_with_diff": 0,
            "pages_detail": pages_info,
        })

    return {
        "mode": "production",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "meta": meta,
        "summary": {
            "total_files": len(files_info),
            "unchanged": len(files_info),
            "changed": 0,
            "added": 0,
            "removed": 0,
            "total_pages": total_pages,
            "diff_pages": 0,
        },
        "files": files_info,
    }


def copy_web_assets(output_dir: Path) -> None:
    """Copy static viewer assets (HTML, JS, CSS) to the report output directory."""
    script_dir = Path(__file__).resolve().parent
    for filename in ["index.html", "report.js", "report.css"]:
        src = script_dir / filename
        if src.exists():
            shutil.copy2(src, output_dir / filename)
        else:
            print(f"Warning: static asset {src} not found.", file=sys.stderr)


def generate_markdown_summary(results: Dict[str, Any], preview_url: Optional[str] = None) -> str:
    """Generate a clean markdown summary table suitable for PR comments."""
    summary = results.get("summary", {})
    total = summary.get("total_files", 0)
    unchanged = summary.get("unchanged", 0)
    changed = summary.get("changed", 0)
    added = summary.get("added", 0)
    removed = summary.get("removed", 0)

    lines = []
    lines.append("### 🔍 hipdf PDF Visual Regression Test Results\n")

    if changed == 0 and added == 0 and removed == 0:
        lines.append(f"✅ **All {total} PDFs match baseline (0 visual differences)**\n")
    else:
        lines.append(f"⚠️ **Visual differences detected across test suite**\n")

    lines.append("| Status | Count |")
    lines.append("| :--- | :--- |")
    lines.append(f"| ✅ Unchanged | {unchanged} |")
    lines.append(f"| ⚠️ Visually Changed | {changed} |")
    if added > 0:
        lines.append(f"| ➕ Added | {added} |")
    if removed > 0:
        lines.append(f"| ➖ Removed | {removed} |")
    lines.append(f"| **Total Evaluated** | **{total}** |\n")

    # If there are changed files, list them
    if changed > 0 or added > 0 or removed > 0:
        lines.append("#### Changed Files Details:\n")
        lines.append("| File | Status | Pages with Diff | Size Diff |")
        lines.append("| :--- | :--- | :--- | :--- |")
        for f in results.get("files", []):
            if f.get("status") in ["changed", "text_changed", "added", "removed"]:
                st_icon = "⚠️ Changed" if f["status"] == "changed" else ("➕ Added" if f["status"] == "added" else "➖ Removed")
                b_size = f.get("baseline_size") or 0
                c_size = f.get("candidate_size") or 0
                diff_kb = (c_size - b_size) / 1024.0
                diff_str = f"{diff_kb:+.1f} KB" if b_size and c_size else "-"
                diff_pages = f.get("pages_with_diff", 0)
                lines.append(f"| `{f['filename']}` | {st_icon} | {diff_pages} | {diff_str} |")
        lines.append("")

    if preview_url:
        lines.append(f"👉 **[View Interactive Side-by-Side Comparison Report]({preview_url})**\n")
        lines.append("_Includes PDF.js side-by-side viewer, swipe slider, diff highlighter, and text inspection._")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="hipdf PDF Visual Regression Tool")
    parser.add_argument("--baseline-dir", type=Path, help="Directory with baseline PDFs (from base branch)")
    parser.add_argument("--candidate-dir", type=Path, help="Directory with candidate PDFs (from PR branch)")
    parser.add_argument("--production-dir", type=Path, help="Directory with production PDFs (for main branch deployment)")
    parser.add_argument("--output-dir", type=Path, required=True, help="Output directory for the static site report")
    parser.add_argument("--pr-number", type=str, default="", help="GitHub Pull Request number")
    parser.add_argument("--pr-title", type=str, default="", help="GitHub Pull Request title")
    parser.add_argument("--base-sha", type=str, default="", help="Base Git commit SHA")
    parser.add_argument("--head-sha", type=str, default="", help="Head Git commit SHA")
    parser.add_argument("--base-ref", type=str, default="main", help="Base branch name")
    parser.add_argument("--head-ref", type=str, default="", help="Head branch name")
    parser.add_argument("--preview-url", type=str, default="", help="Public URL where preview will be hosted")
    parser.add_argument("--dpi", type=int, default=144, help="Rasterization DPI for comparison (default: 144)")
    parser.add_argument("--pixel-threshold", type=int, default=12, help="Color diff threshold (0-255, default: 12)")
    parser.add_argument("--summary-markdown", type=Path, help="Path to write markdown summary for PR comments")
    parser.add_argument("--summary-json", type=Path, help="Path to write JSON summary")
    parser.add_argument("--fail-on-diff", action="store_true", help="Exit with code 1 if visual diff is found")

    args = parser.parse_args()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    meta = {
        "pr_number": args.pr_number,
        "pr_title": args.pr_title,
        "base_sha": args.base_sha[:8] if args.base_sha else "",
        "head_sha": args.head_sha[:8] if args.head_sha else "",
        "base_sha_full": args.base_sha,
        "head_sha_full": args.head_sha,
        "base_ref": args.base_ref,
        "head_ref": args.head_ref,
        "preview_url": args.preview_url,
        "dpi": args.dpi,
    }

    # Production Mode
    if args.production_dir:
        prod_dir = args.production_dir.resolve()
        results = generate_production_results(prod_dir, output_dir, meta)
    elif args.baseline_dir and args.candidate_dir:
        # Comparison Mode
        base_dir = args.baseline_dir.resolve()
        cand_dir = args.candidate_dir.resolve()
        base_pdfs = find_pdf_files(base_dir)
        cand_pdfs = find_pdf_files(cand_dir)

        all_rel_paths = sorted(set(base_pdfs.keys()) | set(cand_pdfs.keys()))
        print(f"Comparing {len(all_rel_paths)} PDF test outputs (DPI: {args.dpi})...")

        files_results = []
        unchanged_count = 0
        changed_count = 0
        added_count = 0
        removed_count = 0
        total_pages = 0
        diff_pages_total = 0

        for rel_path in all_rel_paths:
            base_p = base_pdfs.get(rel_path)
            cand_p = cand_pdfs.get(rel_path)
            res = compare_single_pdf(
                rel_path=rel_path,
                base_path=base_p,
                cand_path=cand_p,
                output_dir=output_dir,
                dpi=args.dpi,
                pixel_threshold=args.pixel_threshold,
            )
            files_results.append(res)

            status = res["status"]
            if status == "unchanged":
                unchanged_count += 1
                status_icon = "✓"
            elif status in ["changed", "text_changed"]:
                changed_count += 1
                status_icon = "⚠"
            elif status == "added":
                added_count += 1
                status_icon = "➕"
            elif status == "removed":
                removed_count += 1
                status_icon = "➖"

            diff_pages = res.get("pages_with_diff", 0)
            diff_pages_total += diff_pages
            total_pages += res.get("candidate_pages") or res.get("baseline_pages") or 0

            print(f"  {status_icon} {rel_path:<45} [{status.upper()}] (diff pages: {diff_pages})")

        results = {
            "mode": "comparison",
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "meta": meta,
            "summary": {
                "total_files": len(files_results),
                "unchanged": unchanged_count,
                "changed": changed_count,
                "added": added_count,
                "removed": removed_count,
                "total_pages": total_pages,
                "diff_pages": diff_pages_total,
            },
            "files": files_results,
        }
    else:
        parser.error("Either --production-dir OR both --baseline-dir and --candidate-dir must be provided.")

    # Write results.json
    results_json_path = output_dir / "results.json"
    with open(results_json_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)

    # Copy static web assets
    copy_web_assets(output_dir)

    # Write summary files if requested
    if args.summary_json:
        args.summary_json.parent.mkdir(parents=True, exist_ok=True)
        with open(args.summary_json, "w", encoding="utf-8") as f:
            json.dump(results["summary"], f, indent=2)

    if args.summary_markdown:
        args.summary_markdown.parent.mkdir(parents=True, exist_ok=True)
        md_content = generate_markdown_summary(results, preview_url=args.preview_url)
        with open(args.summary_markdown, "w", encoding="utf-8") as f:
            f.write(md_content)

    print("\n" + "=" * 60)
    print(f"PDF Regression Complete: {results['summary']['unchanged']} unchanged, {results['summary']['changed']} changed, {results['summary']['added']} added, {results['summary']['removed']} removed")
    print(f"Report generated at: {output_dir}")
    print("=" * 60)

    if args.fail_on_diff and results["summary"]["changed"] > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()

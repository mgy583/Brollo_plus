#!/bin/bash

# 批量将 output 目录下的所有 PDF 文件转换为 PNG 文件
# 使用 ImageMagick 的 magick 命令，设置密度为 300 DPI

# 设置输入和输出目录（这里输入输出都是 output 目录）
INPUT_DIR="output"
OUTPUT_DIR="output"

# 检查 input 目录是否存在
if [ ! -d "$INPUT_DIR" ]; then
  echo "错误：目录 '$INPUT_DIR' 不存在"
  exit 1
fi

# 创建输出目录（如果不存在）
mkdir -p "$OUTPUT_DIR"

# 计数器
count=0
success=0

echo "开始转换 PDF 文件..."

# 遍历 output 目录下所有 PDF 文件
for pdf_file in "$INPUT_DIR"/*.pdf; do
  # 检查是否有 PDF 文件（避免没有匹配文件时的报错）
  if [ ! -f "$pdf_file" ]; then
    echo "警告：在 '$INPUT_DIR' 中没有找到 PDF 文件"
    break
  fi

  # 获取不带路径和扩展名的文件名
  filename=$(basename "$pdf_file" .pdf)

  # 设置输出 PNG 文件路径
  png_file="$OUTPUT_DIR/${filename}.png"

  # 使用 magick 命令转换
  echo "正在转换: $pdf_file -> $png_file"

  if magick -density 300 "$pdf_file" "$png_file"; then
    echo "  ✓ 转换成功"
    ((success++))
  else
    echo "  ✗ 转换失败: $pdf_file"
  fi

  ((count++))
done

echo "================================="
echo "转换完成！"
echo "总共处理: $count 个文件"
echo "成功转换: $success 个文件"
echo "文件保存在: $OUTPUT_DIR 目录"

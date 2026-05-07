const fs = require("fs");
const path = require("path");
const puppeteer = require("puppeteer-core");

const INPUT_DIR = "./html";
const OUTPUT_DIR = "./output";

const chromePaths = [
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/snap/bin/chromium"
];

const executablePath = chromePaths.find(p => fs.existsSync(p));

if (!executablePath) {
    console.error("未找到 Chrome / Chromium");
    process.exit(1);
}

if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR);
}

(async () => {
    const browser = await puppeteer.launch({
        headless: "new",
        executablePath,
        args: [
            "--no-sandbox",
            "--disable-setuid-sandbox"
        ]
    });

    const files = fs.readdirSync(INPUT_DIR)
        .filter(file => file.endsWith(".html"));

    for (const file of files) {
        const page = await browser.newPage();
        const filePath = path.join(INPUT_DIR, file);

        console.log(`正在处理: ${file}`);

        await page.goto(`file://${path.resolve(filePath)}`, {
            waitUntil: "networkidle0"
        });

        await page.waitForSelector("svg");

        // 获取页面真实尺寸
        const bodySize = await page.evaluate(() => {
            return {
                width: document.documentElement.scrollWidth,
                height: document.documentElement.scrollHeight
            };
        });

        await page.pdf({
            path: path.join(
                OUTPUT_DIR,
                `${path.parse(file).name}.pdf`
            ),

            printBackground: true,

            width: `${bodySize.width}px`,
            height: `${bodySize.height}px`,

            pageRanges: "1",
            preferCSSPageSize: true
        });

        console.log(`PDF 导出成功: ${file}`);

        await page.close();
    }

    await browser.close();

    console.log("全部 PDF 导出完成");
})();
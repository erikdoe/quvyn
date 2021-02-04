const { webkit } = require('playwright-webkit');
const browserPromise = webkit.launch();

class DemoPage {
    constructor(page) {
        this.page = page;
    }

    async navigate() {
        await this.page.goto('http://localhost:8080/app/demo.html');
    }

    async heading() {
        return await this.page.innerText('.qv-heading');
    }
    async summaryText() {
        return await this.page.innerText('.qv-comment-section p');
    }

    async firstCommentAuthor() {
        return await this.page.innerText('.qv-author');
    }
    async firstCommentText() {
        return await this.page.innerText('.qv-text p');
    }
    async clickDeleteFirstComment() {
        await this.page.click('text=Delete Comment');
    }

    async fillAuthorName(name) {
        return await this.page.fill('.qv-author-name-field', name);
    }
    async fillComment(comment) {
        await this.page.fill('.qv-text-editor textarea', comment);
    }
    async clickPost() {
        await this.page.click('text=Post Comment');
    }

    async clickPreview() {
        await this.page.click('text=Preview');
    }
    async previewHTML() {
        return await this.page.innerHTML('.qv-text-preview p');
    }

}

// for details on this function see https://playwright.dev/docs/test-runners#ava
async function pageMacro(t, callback) {
    const browser = await browserPromise;
    const page = await browser.newPage();
    const demoPage = new DemoPage(page);
    try {
        await callback(t, demoPage);
    } finally {
        await page.close();
    }
}

module.exports = { pageMacro: pageMacro };

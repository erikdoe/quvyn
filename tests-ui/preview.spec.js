const test = require('ava');
const { pageMacro } = require('./models/page');

test('Preview plain comment', pageMacro, async (t, p) => {

    await p.navigate();
    await p.fillComment('My first comment.');
    await p.clickPreview();
    t.is(await p.previewHTML(), 'My first comment.');

});

test('Preview formatted comment', pageMacro, async (t, p) => {

    await p.navigate();
    await p.fillComment('A **bold** comment.');
    await p.clickPreview();
    t.is(await p.previewHTML(), 'A <strong>bold</strong> comment.');

});

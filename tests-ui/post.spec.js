const test = require('ava');
const { pageMacro } = require('./models/page');

test('Post and delete comments', pageMacro, async (t, p) => {

    await p.navigate();
    t.is(await p.heading(), 'Comments');
    t.is(await p.summaryText(), 'No comments yet');

    await p.fillAuthorName('Joe Bloggs');
    await p.fillComment('My first comment.');
    await p.clickPost();

    t.is(await p.heading(), 'Comments');
    t.is(await p.firstCommentAuthor(), 'Joe Bloggs')
    t.is(await p.firstCommentText(), 'My first comment.')

    await p.fillAuthorName('Joe Bloggs');
    await p.fillComment('Another fine comment.');
    await p.clickPost();

    t.is(await p.heading(), '2 Comments');
    t.is(await p.firstCommentText(), 'My first comment.')

    await p.clickDeleteFirstComment();

    t.is(await p.heading(), 'Comments');
    t.is(await p.firstCommentText(), 'Another fine comment.')

    await p.clickDeleteFirstComment();

    t.is(await p.summaryText(), 'No comments yet');

});

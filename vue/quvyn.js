Vue.component('qv-summary', {
    props: {
        comments: {
            type: Array,
            required: true
        }
    },
    computed: {
        count() {
            return this.comments.length
        }
    },
    template: `
        <div class="qv-summary">
            <p>
                <span v-if="this.count === 0">No comments yet</span>
                <span v-else-if="this.count === 1">1 comment</span>
                <span v-else>{{ this.count }} comments</span>
            </p>
        </div>
    `
})

Vue.component('qv-list', {
    props: {
        comments: {
            type: Array,
            required: true
        },
        dateformat: {
            type: String,
            required: false
        }
    },
    template: `
        <div class="qv-list">
            <ul>
                <li v-for="c in comments">
                    <qv-comment :comment="c" :dateformat="dateformat"></qv-comment>
                </li>
            </ul>
        </div>
    `
})

Vue.component('qv-comment', {
    props: {
        comment: {
            type: Object,
            required: true
        },
        dateformat: {
            type: String,
            required: false
        }
    },
    filters: {
        moment: function (date, format) {
            return moment(date).format(format)
        }
    },
    template: `
        <div class="qv-comment">
            <div class="qv-metadata">
                <div class="qv-author-avatar">
                    <img :src="comment.authorGravatar">
                </div>
                <div class="qv-author-name">
                    <span v-if="comment.authorName">{{ comment.authorName }}</span>
                    <span v-else class="qv-author-anonymous">Anonymous</span>
                </div>
                <div class="qv-timestamp">
                    {{ comment.timestamp | moment(dateformat) }}
                </div>
            </div>
            <div class="qv-text" v-html="comment.textHtml">
           </div>
        </div>
    `
})

Vue.component('qv-comment-editor', {
    data() {
        return {
            name: null,
            email: null,
            error: null
        }
    },
    computed: {
        markdown() {
            return this.$refs.textEditor.markdown
        }
    },
    methods: {
        onSubmit() {
            if (this.markdown) {
                let comment = {
                    path: "/",
                    authorName: this.name,
                    authorEmail: this.email,
                    text: this.markdown,
                }
                this.$emit('post-comment', comment)
                this.name = null
                this.email = null
                this.$refs.textEditor.markdown = null
                this.error = null
            } else {
                this.error = "Please enter some text before posting the comment."
            }
        }
    },
    template: `
        <div class="qv-comment-editor">
            <form class="qv-comment-editor-form" @submit.prevent="onSubmit">
                <div class="qv-input-field qv-author-name-field">
                    <input id="name" v-model="name" placeholder="Your name (optional)"> 
                </div>
                <div class="qv-input-field qv-author-email-field">
                    <input id="name" v-model="email" placeholder="Your email address (optional)"> 
                </div>
                <qv-text-editor ref="textEditor"></qv-text-editor>
                <div class="qv-submit-error" v-if="error">
                    {{ error }}
                </div>
                <div class="qv-submit">
                    <input type="submit" value="Post comment">
                </div>
            </form>
        </div>
    `
})

Vue.component('qv-text-editor', {
    data() {
        return {
            showingPreview: false,
            markdown: null,
            preview: '(Nothing to preview)'
        }
    },
    methods: {
        showTextArea() {
            this.showingPreview = false
        },
        showPreview() {
            fetch('/preview', {
                method: 'POST',
                headers: {'Content-Type': 'application/json;charset=utf-8'},
                body: JSON.stringify({text: this.markdown})
            })
                .then(response => response.text())
                .then(text => this.preview = text ? text : '(Nothing to preview)')
            this.showingPreview = true
        },
    },
    template: `
        <div class="qv-text-editor">
            <span class="qv-tab" :class="{ 'qv-active-tab': !showingPreview }" @click="showTextArea()">Write</span>
            <span class="qv-tab" :class="{ 'qv-active-tab': showingPreview }" @click="showPreview()">Preview</span>
            <div class="qv-tab-content qv-textarea-field" v-show="!showingPreview">
                <textarea id="text" v-model="markdown" cols="80" rows="14"></textarea> 
            </div>
            <div class="qv-tab-content" v-show="showingPreview">
                <div class="qv-text-preview" v-html="preview"></div>
            </div>
        </div>
    `
})


class Quvyn {
    constructor(element, baseurl) {
        this.app = new Vue({
            el: element,
            data: {
                comments: []
            },
            methods: {
                postComment(comment) {
                    fetch('/comments', {
                        method: 'POST',
                        headers: {'Content-Type': 'application/json;charset=utf-8'},
                        body: JSON.stringify(comment)
                    })
                        .then(response => response.json())
                        .then(json => this.comments.push(json))
                }
            },
            created() {
                fetch(baseurl + "/comments?p=%2f")
                    .then(response => response.json())
                    .then(json => this.comments = json.comments)
            }
        })
    }
}


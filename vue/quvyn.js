
Vue.component('qv-comment-section', {
    props: {
        baseurl: {
            type: String,
            required: true
        }
    },
    methods: {
        postComment(comment) {
            fetch(this.baseurl + '/comments', {
                method: 'POST',
                headers: {'Content-Type': 'application/json;charset=utf-8'},
                body: JSON.stringify(comment)
            })
                .then(response => response.json())
                .then(json => this.comments.push(json) )
        },
        getPreview(markdown) {
            this.preview = "<i>(loading preview)</i>"
            fetch(this.baseurl + '/preview', {
                method: 'POST',
                headers: {'Content-Type': 'application/json;charset=utf-8'},
                body: JSON.stringify({text: markdown})
            })
                .then(response => response.text())
                .then(text => this.preview = text)
        }
    },
    data() {
        return {
            comments: [],
            preview: ''
        }
    },
    created() {
        fetch(this.baseurl + "/comments?p=%2f")
            .then(response => response.json())
            .then(json => this.comments = json.comments)
    },
    template: `
        <section class="qv-comment-section">
            <h2>Comments</h2>
            <qv-summary :comments="comments"></qv-summary>
            <qv-list :comments="comments"></qv-list>
            <h3>Leave your comment</h3>
            <qv-comment-editor :preview="preview" @post-comment="postComment" @get-preview="getPreview"></qv-comment-editor>
        </section>
    `
})


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
        }
    },
    template: `
        <div class="qv-list">
            <ul>
                <li v-for="c in comments">
                    <qv-comment :comment="c"></qv-comment>
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
        }
    },
    filters: {
        formatTimestamp: function (text) {
            return luxon.DateTime.fromISO(text).toLocaleString(luxon.DateTime.DATETIME_MED)
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
                    {{ comment.timestamp | formatTimestamp() }}
                </div>
            </div>
            <div class="qv-text" v-html="comment.textHtml">
           </div>
        </div>
    `
})


Vue.component('qv-comment-editor', {
    props: {
        preview: {
            type: String,
            required: true
        }
    },
    methods: {
        showTextArea() {
            this.showingPreview = false
        },
        showPreview() {
            this.$emit('get-preview', this.markdown)
            this.showingPreview = true
        },
        submitForm() {
            if (this.markdown) {
                let comment = {
                    path: "/",
                    authorName: this.name,
                    authorEmail: this.email,
                    text: this.markdown,
                }
                this.$emit('post-comment', comment)
                // TODO: nulling here will cause an issue for the user if the post fails
                this.name = null
                this.email = null
                this.markdown = null
                this.error = null
            } else {
                this.error = "Please enter some text before posting the comment."
            }
        }
    },
    data() {
        return {
            name: null,
            email: null,
            markdown: null,
            showingPreview: false,
            error: null
        }
    },
    template: `
        <div class="qv-comment-editor">
            <form class="qv-comment-editor-form" @submit.prevent="submitForm">
                <div class="qv-input-field qv-author-name-field">
                    <input id="name" v-model="name" placeholder="Your name (optional)"> 
                </div>
                <div class="qv-input-field qv-author-email-field">
                    <input id="name" v-model="email" placeholder="Your email address (optional)"> 
                </div>
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


Vue.component('qv-comment-section', {
    props: {
        baseurl: {
            type: String,
            required: true
        }
    },
    methods: {
        postComment(comment) {
            var location = null
            fetch(this.baseurl + '/comments', {
                method: 'POST',
                headers: {'Content-Type': 'application/json;charset=utf-8'},
                body: JSON.stringify(comment)
            })
                .then(response => {
                    location = response.headers.get("location")
                    return response.json()
                })
                .then(json => {
                    localStorage.setItem(json.idh, location)
                    this.comments.push(json)
                })
        },
        deleteComment(idh) {
            let location = localStorage.getItem(idh)
            if (!location) {
                return
            }
            fetch(this.baseurl + location, {
                method: 'DELETE',
            })
                .then(response => {
                    if (response.status === 200) {
                        index = this.comments.findIndex(c => c.idh === idh)
                        if (index > -1) {
                            this.comments.splice(index, 1)
                        }
                    }
                })
        },
        getPreview(markdown) {
            this.preview = "<i>(loading preview)</i>"
            fetch(this.baseurl + '/preview', {
                method: 'POST',
                headers: {'Content-Type': 'application/json;charset=utf-8'},
                body: JSON.stringify({text: markdown})
            })
                .then(response => response.text())
                .then(text => this.preview = text ? text : '')
        }
    },
    created() {
        let p = encodeURIComponent(window.location.pathname)
        fetch(this.baseurl + "/comments?p=" + p)
            .then(response => response.json())
            .then(json => this.comments = json.comments)
    },
    data() {
        return {
            comments: [],
            preview: ''
        }
    },
    template: `
        <section class="qv-comment-section">
            <qv-heading :comments="comments"></qv-heading>
            <p v-if="this.comments.length === 0">No comments yet</p>
            <qv-list :comments="comments" @delete-comment="deleteComment"></qv-list>
            <qv-comment-editor :preview="preview" @post-comment="postComment" @get-preview="getPreview"></qv-comment-editor>
        </section>
    `
})


Vue.component('qv-heading', {
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
        <h2 class="qv-heading">
            <span v-if="this.count === 0">Comments</span>
            <span v-else-if="this.count === 1">Comments</span>
            <span v-else>{{ this.count }} Comments</span>
        </h2>
    `
})


Vue.component('qv-list', {
    props: {
        comments: {
            type: Array,
            required: true
        }
    },
    filters: {
        formatTimestamp: function (text) {
            return luxon.DateTime.fromISO(text).toLocaleString(luxon.DateTime.DATETIME_MED)
        }
    },
    methods: {
        getFromStorage(key) {
            return localStorage.getItem(key)
        },
        submitForm(idh) {
            this.$emit('delete-comment', idh)
        }
    },
    template: `
        <ul class="qv-list">
            <li v-for="comment in comments" class="qv-comment">
                <div class="qv-metadata">
                    <div class="qv-avatar">
                        <img :src="comment.authorGravatar + '?r=pg&s=40'">
                    </div>
                    <div class="qv-author">
                        <span v-if="comment.authorName">{{ comment.authorName }}</span>
                        <span v-else class="qv-author-anonymous">Anonymous</span>
                    </div>
                    <div class="qv-timestamp">
                        {{ comment.timestamp | formatTimestamp() }}
                    </div>
                </div>
                <div class="qv-text" v-html="comment.textHtml">
                </div>
                <div v-if="getFromStorage(comment.idh)">
                    <form class="qv-delete-form" @submit.prevent="submitForm(comment.idh)">
                        <input class="qv-submit" type="submit" value="Delete comment">
                    </form>
                </div>
            </li>
        </ul>
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
            if (this.markdown) {
                this.previewStyle.width = this.$refs.textarea.offsetWidth - 10 + "px"
                this.previewStyle.height = (this.$refs.textarea.offsetHeight - 10) + "px"
                this.$emit('get-preview', this.markdown)
                this.showingPreview = true
            }
        },
        submitForm() {
            if (this.markdown) {
                let comment = {
                    path: window.location.pathname,
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
                this.showingPreview = false
            } else {
                this.error = "Please write something before posting the comment."
            }
        }
    },
    data() {
        return {
            name: null,
            email: null,
            markdown: null,
            showingPreview: false,
            previewStyle: { width: "100%", height: "100px" },
            error: null
        }
    },
    template: `
        <div class="qv-comment-editor">
            <h2 class="qv-editor-heading">Leave your comment</h2>
            <p class="qv-editor-help">Your email address is used only for displaying your Gravatar. It won't be displayed in 
            the comment and other people won't see it. Comments can be styled with <a href="http://daringfireball.net/projects/markdown/basics">markdown</a>.
            Fenced code blocks are supported.</p>
            <form class="qv-comment-editor-form" @submit.prevent="submitForm">
                <input class="qv-input-field qv-author-name-field" id="name" v-model="name" placeholder="Your name (optional)"> 
                <input class="qv-input-field qv-author-email-field" id="name" v-model="email" placeholder="Your email address (optional, for Gravatar
 only)"> 
                <div class="qv-text-editor">
                    <span class="qv-tab" :class="{ 'qv-active-tab': !showingPreview }" @click="showTextArea()">Write</span>
                    <span class="qv-tab" :class="{ 'qv-active-tab': showingPreview }" @click="showPreview()">Preview</span>
                    <div class="qv-tab-content qv-textarea-field" v-show="!showingPreview">
                        <textarea id="text" v-model="markdown" cols="80" rows="14" ref="textarea" style="width: 100%"></textarea> 
                    </div>
                    <div class="qv-tab-content" v-show="showingPreview">
                        <div class="qv-text-preview" v-html="preview" v-bind:style="previewStyle"></div>
                    </div>
                </div>
                <div class="qv-submit-error" v-if="error">
                    {{ error }}
                </div>
                <input class="qv-submit" type="submit" value="Post comment">
            </form>
        </div>
    `
})

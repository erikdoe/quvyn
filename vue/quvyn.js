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
            return moment(date).format(format);
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
            <div class="qv-content" v-html="comment.contentHtml">
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
            created() {
                fetch(baseurl + "/comments?p=%2f2%2f")
                    .then(response => response.json())
                    .then(json => {
                        this.comments = json.comments
                    })
            }
        })
    }
}


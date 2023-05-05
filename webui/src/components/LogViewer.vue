<template>
    <v-sheet ref="scroll" class="logview px-4 py-2" height="300" color="background">
        <div v-for="item in logs" :key="item.timestamp">
            <pre><code><span class="timestamp">{{ time(item.timestamp) }}</span> <span class="level">{{ item.level }}</span> <span>{{ item.message }}</span> <span class="fields">{{ fields(item.fields) }}</span></code></pre>
        </div>
    </v-sheet>
</template>

<script setup>
import { defineProps, watch, ref } from 'vue';
const props = defineProps({
    logs: {
        type: Array,
        default: [],
    },
});

const scroll = ref(null);

watch(
    () => props.logs,
    () => {
        setTimeout(() => {
            if (scroll.value) {
                scroll.value.$el.scrollTop = scroll.value.$el.scrollHeight;
            }
        }, 0)
    }
);

function time(timestamp) {
    return new Date(timestamp * 1000).toLocaleString();
}

function fields(obj) {
    return Object.entries(obj).map(([key, value]) => `${key}=${value}`).join(' ');
}

</script>

<style scoped>
.logview {
    overflow-y: scroll;
    overflow-x: auto;
}

span.timestamp {
    display: inline-block;
    margin-right: 5px;
    color: burlywood;
}

span.level {
    display: inline-block;
    width: 50px;
    text-transform: uppercase;
    color: lightskyblue;
}

span.fields {
    display: inline-block;
    margin-right: 5px;
    color: gray;
}
</style>
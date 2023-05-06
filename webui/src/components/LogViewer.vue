<template>
    <v-sheet ref="scroll" class="logview" height="300" color="background">
        <v-table density="compact">
            <tbody>
                <tr v-for="item in logs" :key="item.timestamp">
                    <td class="timestamp">{{ time(item.timestamp) }}</td>
                    <td class="level" :data-level="item.level">{{ item.level }}</td>
                    <td>{{ item.message }} <span class="fields">{{ fields(item.fields) }}</span></td>
                </tr>
            </tbody>
        </v-table>
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
.v-table__wrapper {
    overflow: visible;
}

.logview {
    overflow-y: scroll;
    overflow-x: auto;
}

.logview .v-table {
    background-color: transparent;
    white-space: nowrap;
    font-family: monospace;
    font-size: 0.8em;
}

.logview .v-table td {
    border: none;
}

td.timestamp {
    color: burlywood;
}

td.level {
    text-transform: uppercase;
}

td.level[data-level="trance"] {
    color: lightgray;
}

td.level[data-level="debug"] {
    color: lightcoral;
}

td.level[data-level="info"] {
    color: green;
}

td.level[data-level="warn"] {
    color: orange;
}

td.level[data-level="error"] {
    color: red;
}

span.fields {
    display: inline-block;
    margin-right: 5px;
    color: gray;
}
</style>
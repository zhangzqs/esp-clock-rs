<script setup lang="ts">
import { click, clicks } from './api/index'
</script>

<template>
    <button @click="click">单击</button>
    <button @click="() => clicks(2)">双击</button>
    <router-view></router-view>
</template>

<style scoped></style>

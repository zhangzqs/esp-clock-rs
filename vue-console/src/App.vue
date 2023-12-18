<script setup lang="ts">
import { Axios } from 'axios'
const login = () => {
    const axios = new Axios()
    axios.get('/api/login')

    console.log('login')
}
</script>

<template>
    <button @click="login">登录</button>
    <router-view></router-view>
</template>

<style scoped></style>

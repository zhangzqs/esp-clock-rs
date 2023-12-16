<template>
  <div class="home">
    <img alt="Vue logo" src="../assets/logo.png" />
    <button @click="count++">count is: {{ count }}</button>
    <button @click="getPosts()">Get Posts</button>
    <input type="text" v-model="postText" />
    <HelloWorld msg="Welcome to Your Vue.js + TypeScript App" />
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import HelloWorld from "@/components/HelloWorld.vue"; // @ is an alias to /src

import { Axios } from "axios";

const axios = new Axios({
  timeout: 1000,
});

export default defineComponent({
  name: "HomeView",
  components: {
    HelloWorld,
  },
  methods: {
    async getPosts() {
      const response = await axios.get("/posts");
      console.log(response);
      this.postText = response.data[0].title;
    },
  },
  data() {
    return {
      count: ref(0),
      postText: ref(""),
    };
  },
});
</script>

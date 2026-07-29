<script setup lang="ts">
// 项目卡片列表（骨架版本，接入后端 list_projects）
import { computed, onMounted, ref } from "vue";
import { NEmpty, NSpin } from "naive-ui";
import { checkProjects, listProjects } from "../api";
import type { Project } from "../types";
import ProjectCard from "./ProjectCard.vue";

const props = defineProps<{ search: string }>();

const projects = ref<Project[]>([]);
const invalidIds = ref<Set<string>>(new Set());
const loading = ref(false);

async function refresh() {
  loading.value = true;
  try {
    projects.value = await listProjects();
    const statuses = await checkProjects();
    invalidIds.value = new Set(
      statuses.filter((s) => !s.exists).map((s) => s.id),
    );
  } catch (e) {
    projects.value = [];
    invalidIds.value = new Set();
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

// 排序 + 搜索过滤
const visibleProjects = computed(() => {
  const kw = props.search.trim().toLowerCase();
  const filtered = kw
    ? projects.value.filter((p) => p.name.toLowerCase().includes(kw))
    : projects.value.slice();

  filtered.sort((a, b) => {
    if (a.starred !== b.starred) return a.starred ? -1 : 1;
    return b.last_accessed - a.last_accessed;
  });
  return filtered;
});

defineExpose({ refresh });
</script>

<template>
  <div class="project-list">
    <NSpin v-if="loading" size="small" />
    <NEmpty
      v-else-if="!visibleProjects.length"
      description="暂无项目，点击右下角『＋』添加"
    />
    <template v-else>
      <ProjectCard
        v-for="p in visibleProjects"
        :key="p.id"
        :project="p"
        :invalid="invalidIds.has(p.id)"
        @refresh="refresh"
      />
    </template>
  </div>
</template>

<style scoped>
.project-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 72px;
}
</style>

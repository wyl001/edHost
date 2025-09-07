<script lang="ts" setup>
import {onMounted, reactive, ref} from "vue";
import {invoke} from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import {BaseDirectory, exists} from "@tauri-apps/plugin-fs"
import {openPath} from "@tauri-apps/plugin-opener"

interface HostEntry {
  ip: string;
  hostname: string;
  enabled: boolean;
}

const hostForm = reactive({
  ip: '',
  hostname: ''
});

const hosts = ref<HostEntry[]>([]);
const status = ref('就绪');

const addHostEntry = async () => {

  if (!isValidIPv4(hostForm.ip) || !isValidDomain(hostForm.hostname)) {
    await message("IP地址或域名格式不正确！", {
      title: '格式错误',
      kind: 'error'
    });
    return;
  }

  try {
    // 调用Tauri命令添加主机记录
    await invoke('add_host_entry', { hostForm });

    hosts.value.push({
      ip: hostForm.ip,
      hostname: hostForm.hostname,
      enabled: true
    });

    hostForm.ip = '';
    hostForm.hostname = '';
    status.value = '已添加新映射';
    await message("主机记录添加成功！", {
      title: '成功',
      kind: 'info'
    });
  } catch (error) {
    await message(`添加失败: ${error}`, {
      title: '错误',
      kind: 'error'
    });
  }
};

const openFile = async () =>{
  await invoke('open_file')
}

const removeHostEntry = (index: number) => {
  hosts.value.splice(index, 1);
  status.value = '已删除映射';
};

const toggleHostEntry = (index: number) => {
  hosts.value[index].enabled = !hosts.value[index].enabled;
  status.value = hosts.value[index].enabled ? '已启用映射' : '已禁用映射';
};

const saveChanges = async () => {
  try {
    status.value = '保存中...';
    // 这里调用Tauri命令保存hosts文件
    await invoke('save_hosts', { entries: hosts.value });
    status.value = '保存成功';
  } catch (error) {
    status.value = '保存失败';
    console.error('保存失败:', error);
  }
};

const backupHosts = async () => {
  try {
    status.value = '备份中...';
    const bak_path = await invoke('backup_hosts');
    status.value = '备份成功';
    console.log(bak_path)
    alert("备份成功,地址："+bak_path)
    return;
  } catch (error) {
    status.value = '备份失败';
    console.error('备份失败:', error);
    alert('备份失败:'+error)
  }
};


const reloadHosts = async () => {
  try {
    status.value = '刷新中...';
    hosts.value = await invoke<HostEntry[]>('load_hosts');
    status.value = '刷新成功';
    await message("刷新成功",{title:"成功",kind:"info"})
  } catch (error) {
    status.value = '刷新失败';
    console.error('刷新失败:', error);
    await message("刷新失败",{title:"成功",kind:"info"})
  }
};

const load_once = async () =>{
  hosts.value = await invoke<HostEntry[]>('load_hosts');
}

function isValidIPv4(ip: string): boolean {
  const ipv4Regex =
      /^(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)){3}$/;
  return ipv4Regex.test(ip);
}
function isValidDomain(domain: string): boolean {
  const domainRegex =
      /^(?=.{1,253}$)(?!-)([a-zA-Z0-9-]{1,63}(?<!-)\.)+[a-zA-Z]{2,}$/;
  return domainRegex.test(domain);
}

onMounted(() => {
  load_once();
});
</script>

<template>
  <main class="container">
    <header>
      <h1>Host文件管理器</h1>
      <div>
        <button class="btn btn-info" @click="openFile">打开文件</button>
        <button class="btn btn-warning" @click="backupHosts">备份</button>
        <button class="btn btn-primary" @click="reloadHosts">刷新</button>
      </div>
    </header>
    
    <div class="main-content">
      <div class="card hosts-card">
        <div class="card-header">
          <h2>当前Host条目 ({{ hosts.length }})</h2>
        </div>
        <div class="card-body">
          <div class="table-container">
            <table>
              <thead>
              <tr>
                <th>状态</th>
                <th>IP地址</th>
                <th>主机名</th>
                <th>操作</th>
              </tr>
              </thead>
              <tbody>
              <tr v-for="(entry, index) in hosts" :key="index">
                <td>
                  <span :class="{ 'text-success': entry.enabled, 'text-muted': !entry.enabled }">
                    {{ entry.enabled ? '✓' : '✗' }}
                  </span>
                </td>
                <td>{{ entry.ip }}</td>
                <td>{{ entry.hostname }}</td>
                <td>
                  <button :class="entry.enabled ? 'btn-warning' : 'btn-success'" class="btn btn-sm" @click="toggleHostEntry(index)">
                    {{ entry.enabled ? '禁用' : '启用' }}
                  </button>
                  <button class="btn btn-sm btn-danger" @click="removeHostEntry(index)">删除</button>
                </td>
              </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="status-bar">
          <span>{{ status }}</span>
          <button class="btn btn-primary" @click="saveChanges">保存更改</button>
        </div>
      </div>

      <div class="card add-card">
        <div class="card-header">
          <h2>添加新映射</h2>
        </div>
        <div class="card-body">
          <form @submit="addHostEntry">
            <div class="form-group">
              <label for="ip">IP地址</label>
              <input id="ip" v-model="hostForm.ip" placeholder="例如: 127.0.0.1" required type="text">
            </div>
            <div class="form-group">
              <label for="hostname">主机名</label>
              <input id="hostname" v-model="hostForm.hostname" placeholder="例如: example.local" required type="text">
            </div>
            <button class="btn btn-success" type="submit">添加</button>
          </form>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 10px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding: 12px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

header h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.btn {
  margin-left: 8px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-weight: 500;
  font-size: 0.9rem;
  transition: all 0.2s ease;
  cursor: pointer;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover {
  background: #0056b3;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.3);
}

.btn-warning {
  background: #ffc107;
  color: #212529;
}

.btn-warning:hover {
  background: #e0a800;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(255, 193, 7, 0.3);
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-success:hover {
  background: #218838;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.card {
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  margin-bottom: 16px;
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.12);
}

.card-header {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  padding: 12px 16px;
  border-bottom: 1px solid #dee2e6;
}

.card-header h2 {
  margin: 0;
  color: #495057;
  font-size: 1.2rem;
  font-weight: 600;
}

.card-body {
  padding: 16px;
}

.form-group {
  margin-bottom: 12px;
  text-align: left;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: #495057;
}

.form-group input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  font-size: 0.9rem;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.form-group input:focus {
  outline: none;
  border-color: #007bff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 20px;
}

th, td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid #dee2e6;
  font-size: 0.9rem;
}

th {
  background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
  font-weight: 600;
  color: #495057;
}

tr:hover {
  background-color: #f8f9fa;
}

.text-success {
  color: #28a745;
}

.text-muted {
  color: #6c757d;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 0.8rem;
  margin-right: 4px;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover {
  background: #c82333;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(220, 53, 69, 0.3);
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f8f9fa;
  border-top: 1px solid #dee2e6;
  font-size: 0.9rem;
}

#statusInfo {
  color: #6c757d;
  font-weight: 500;
}

.main-content {
  display: grid;
  grid-template-columns: 1fr 350px;
  gap: 16px;
  align-items: start;
}

.hosts-card {
  grid-column: 1;
}

.add-card {
  grid-column: 2;
  position: sticky;
  top: 20px;
}

.table-container {
  max-height: 400px;
  overflow-y: auto;
}

@media (max-width: 1024px) {
  .main-content {
    grid-template-columns: 1fr;
  }
  
  .add-card {
    position: static;
  }
}

@media (max-width: 768px) {
  .container {
    padding: 8px;
  }
  
  header {
    flex-direction: column;
    gap: 12px;
    text-align: center;
    margin-bottom: 12px;
  }
  
  header h1 {
    font-size: 1.3rem;
  }
  
  .card-body {
    padding: 12px;
  }
  
  table {
    font-size: 0.85rem;
  }
  
  th, td {
    padding: 6px 8px;
  }
  
  .btn {
    padding: 6px 12px;
    font-size: 0.85rem;
    margin-left: 6px;
  }
}
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>
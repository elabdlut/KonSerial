import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/serial'
    },
    {
      path: '/serial',
      name: 'Serial',
      component: () => import('../views/SerialView.vue'),
      meta: { title: '串口调试' }
    },
    {
      path: '/chart',
      name: 'Chart',
      component: () => import('../views/ChartView.vue'),
      meta: { title: '波形图' }
    },
    {
      path: '/network',
      name: 'Network',
      component: () => import('../views/NetworkView.vue'),
      meta: { title: '网络调试' }
    },
    {
      path: '/script',
      name: 'Script',
      component: () => import('../views/ScriptView.vue'),
      meta: { title: '脚本编辑' }
    },
    {
      path: '/history',
      name: 'History',
      component: () => import('../views/HistoryView.vue'),
      meta: { title: '历史记录' }
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('../views/SettingsView.vue'),
      meta: { title: '设置' }
    }
  ]
})

router.afterEach((to) => {
  const title = to.meta.title as string | undefined
  if (title) {
    document.title = `${title} - KonSerial`
  }
})

export default router

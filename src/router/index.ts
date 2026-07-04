import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/screw-spec'
    },
    {
      path: '/screw-spec',
      name: 'ScrewSpec',
      component: () => import('../views/ScrewSpec.vue')
    },
    {
      path: '/punch',
      name: 'Punch',
      component: () => import('../views/Punch.vue')
    },
    {
      path: '/die',
      name: 'Die',
      component: () => import('../views/Die.vue')
    },
    {
      path: '/belt',
      name: 'Belt',
      component: () => import('../views/Belt.vue')
    },
    {
      path: '/main-mold',
      name: 'MainMold',
      component: () => import('../views/MainMold.vue')
    },
    {
      path: '/scissor',
      name: 'Scissor',
      component: () => import('../views/Scissor.vue')
    },
    {
      path: '/upper-punch',
      name: 'UpperPunch',
      component: () => import('../views/UpperPunch.vue')
    },
    {
      path: '/settings',
      name: 'Settings',
      component: () => import('../views/Settings.vue')
    }
  ]
})

export default router

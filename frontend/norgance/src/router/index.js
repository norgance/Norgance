import Vue from 'vue';
import VueRouter from 'vue-router';
import Home from '../views/Home.vue';

Vue.use(VueRouter);

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
  },
  {
    path: '/about',
    name: 'About',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "about" */ '../views/About.vue'),
  },
  {
    path: '/register-citizenship',
    name: 'RegisterCitizenship',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "register-citizenship" */ '../views/RegisterCitizenship.vue'),
    children: [
      {
        path: 'names',
        name: 'CitizenApplicationNames',
        component: () => import(/* webpackChunkName: "citizen-application-names" */ '../views/citizenApplication/Names.vue'),
      },
      {
        path: 'birthday',
        name: 'CitizenApplicationBirthday',
        component: () => import(/* webpackChunkName: "citizen-application-birthday" */ '../views/citizenApplication/Birthday.vue'),
      },
      {
        path: 'identifier',
        name: 'CitizenApplicationIdentifier',
        component: () => import(/* webpackChunkName: "citizen-application-identifier" */ '../views/citizenApplication/Identifier.vue'),
      },
      {
        path: 'password',
        name: 'CitizenApplicationPassword',
        component: () => import(/* webpackChunkName: "citizen-application-password" */ '../views/citizenApplication/Password.vue'),
      },
      {
        path: 'summary',
        name: 'CitizenApplicationSummary',
        component: () => import(/* webpackChunkName: "citizen-application-summary" */ '../views/citizenApplication/Summary.vue'),
      },
      {
        path: '',
        name: 'CitizenApplicationStart',
        component: () => import(/* webpackChunkName: "citizen-application-start" */ '../views/citizenApplication/Start.vue'),
      },
    ],
  },
];

const router = new VueRouter({
  routes,
});

export default router;

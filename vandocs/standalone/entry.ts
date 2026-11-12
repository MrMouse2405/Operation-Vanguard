import { mount } from 'svelte';
import App from './App.svelte';
import '../src/routes/mail.css';
mount(App, { target: document.getElementById('app')! });

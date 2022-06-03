import { createSignal, createEffect, Show } from "solid-js";

import { Routes, Route, Link } from "solid-app-router"

import styles from './App.module.css';

import "bootstrap/dist/css/bootstrap.min.css";
import "../src/assets/css/Login-Form-Dark.css";

import "../src/assets/bootstrap/css/bootstrap.min.css";
import "../src/assets/fonts/ionicons.min.css";
import "../src/assets/css/Article-List.css";
import "../src/assets/css/Contact-Form-Clean.css";
import "../src/assets/css/Navigation-Clean.css";
import "../src/assets/css/Navigation-with-Button.css";
import "../src/assets/css/Registration-Form-with-Photo.css";
import "../src/assets/css/styles.css";

const [text, setText] = createSignal("");
const [count, setCount] = createSignal(0);
const [path, setPath] = createSignal("/");
const [url, setUrl] = createSignal("http://127.0.0.1:44444/api/manage/dump_to_json");
const [apiKey, setApiKey] = createSignal("OJQ6rCDXRyIj498hTjSIsG+Kkl1xeGUoV3zHhulHCg0=");

function make_request() {
    window.history.pushState("", "", "/>> -- <<");
    const otherParams = {
        headers: {
            "Authorization": "Bearer " + apiKey(),
        },
        method: "GET"
    }
    fetch(url(), otherParams)
        .then(data => {
                return data.text();
            })
        .then(res => {
                setText(res);
            })
        .catch(error => console.log(error));
}

function inc() {
    setCount(count() + 1);
    window.history.pushState("", "", "/counted/" + count().toString());
}

function dec() {
    setCount(count() - 1);
    window.history.pushState("", "", "/counted/" + count().toString());
}

function login() {
    let formelement = document.getElementById("LoginForm");
    console.log(formelement);
}

function Legacy() {
    return (
        <div class={styles.App}>
            <header class={styles.header}>
            <div class="input-group input-group-sm mb-3" style="width: 500px">
                <input 
                    type="text"
                    class="form-control"
                    aria-label="Small"
                    aria-describedby="inputGroup-sizing-sm"
                    value={apiKey()}
                    onInput={e => setApiKey(e.currentTarget.value)}>
                </input>
            </div>
            <div class="input-group input-group-sm mb-3" style="width: 500px">
                <input 
                    type="text"
                    class="form-control"
                    aria-label="Small"
                    aria-describedby="inputGroup-sizing-sm"
                    value={url()}
                    onInput={e => setUrl(e.currentTarget.value)}>
                </input>
            </div>
                <button 
                    class="btn btn-success"
                    onClick={() => {make_request()}}
                >
                    Make request
                </button>
                <button 
                    class="btn btn-warning"
                    onClick={() => {inc()}}
                >
                    Increment
                </button>
                <button 
                    class="btn btn-danger"
                    onClick={() => {dec()}}
                >
                    Decrement
                </button>
                <button 
                    class="btn btn-primary"
                    onClick={() => {movePath("/login")}}
                >
                    Login
                </button>
                <code style="white-space: pre-wrap; text-align: justify;">
                    {text()}
                </code>
            </header>
        </div>
    )
}

function movePath(newPath: String) {
    window.history.pushState("", "", newPath.toString())
    setPath(newPath.toString());
    return (<></>);
}



function DashboardPage() {
    return (
        <>
            Dashboard Page
        </>
    )
}

function LoginPage() {
    return (
        <section class="login-dark" style="max-height: 100vh;">
            <form id="LoginForm">
                <h2 class="visually-hidden">Login Form</h2>
                <div class="illustration"><i class="icon ion-ios-locked-outline"></i></div>
                <div class="mb-3"><input class="form-control" type="email" name="email" placeholder="Email"></input></div>
                <div class="mb-3"><input class="form-control" type="password" name="password" placeholder="Пароль"></input></div>
                <div class="mb-3">
                    <button class="btn btn-primary d-block w-100" onClick={() => login()}>
                        Войти
                    </button>
                </div>
                <button class="btn btn-link forgot" onClick={() => movePath("/contact")}>
                    Не можете войти?
                </button>
            </form>
        </section>
    );
}

function ContactPage() {
    return (
        <>
            Contact Page
        </>
    )
}

function AdminPage() {
    return (
        <>
            Admin Page
        </>
    )
}

function SettingsPage() {
    return (
        <>
            Settings Page
        </>
    )
}

function App() {
    console.log(location);
    setPath(location.pathname);

    addEventListener('popstate', () => {setPath(location.pathname)});

    document.getElementById('LoginForm').addEventListener('submit', (event) => {
        // stop form submission
        event.preventDefault();
    });

    return (
        <div>
            <Show when={path() == "/"}>
                {movePath("/dashboard")}
            </Show>
            <Show when={path() == "/dashboard"}>
                <DashboardPage />
            </Show>
            <Show when={path() == "/login"}>
                <LoginPage />
            </Show>
            <Show when={path() == "/contact"}>
                <ContactPage />
            </Show>
            <Show when={path() == "/admin"}>
                <AdminPage />
            </Show>
            <Show when={path() == "/settings"}>
                <SettingsPage />
            </Show>
            {/* <Routes>
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="/login" element={<Login />} />
                <Route path="/contact" element={<Contact />} />
                <Route path="/admin" element={<Admin />} />
                <Route path="/settings" element={<Settings />} />
            </Routes> */}
        </div>
    )
};

export default App;

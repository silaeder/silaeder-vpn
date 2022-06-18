import { createSignal, onMount, Show } from "solid-js";

import styles from './App.module.css';

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

let form;

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


function movePath(newPath) {
    window.history.pushState("", "", newPath)
    setPath(newPath)
    return (<></>);
}

function sendForm() {
    const XHR = new XMLHttpRequest();
    const FD = new FormData( form );
    console.log(FD.get("email"));
    console.log(FD.get("password"));
    XHR.onreadystatechange = function() {
        if (XHR.readyState == XMLHttpRequest.DONE) {
            console.log(XHR.getResponseHeader("token"));
        }
    }
    XHR.open( "POST", "https://justdprroz.ru/auth/login" );
    XHR.send( FD );
    console.log("sent");
}

function DashboardPage() {
    return (
        <>
            Dashboard Page
        </>
    )
}

function LoginPage() {

    onMount(() => {
        form.addEventListener( "submit", function ( event ) {
            event.preventDefault();
            sendForm();
        })
    });

    return (
        <section class="login-dark" style="max-height: 100vh;">
            <form id="LoginForm" ref={form}>
                <h2 class="visually-hidden">Login Form</h2>
                <div class="illustration"><i class="icon ion-ios-locked-outline"></i></div>
                <div class="mb-3">
                    <input id="myemail" class="form-control" type="email" name="email" placeholder="Email" />
                </div>
                <div class="mb-3">
                    <input id="mypassword" class="form-control" type="password" name="password" placeholder="Пароль" />
                </div>
                <div class="mb-3">
                    <button class="btn btn-primary d-block w-100" >
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

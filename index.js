import init, { hello } from "./pkg/test.js"

async function run() {
    await init()
    const pem = "123";
    const num = "20123123";
    
    console.log(hello("123"));
    document.body.textContent = hello("123")
};

run()
import {DeftWindow} from "deft-react";
import App from "./app";
import React from "react";

function initWindow(): DeftWindow {
    const window = globalThis.mainWindow || (globalThis.mainWindow = new DeftWindow({
        title: 'DeftVideoPlayer',
        decorations: false,
        width: 800,
        height: 500,
    }));
    window.bindResize((e: IResizeEvent) => {
        console.log("window resized", e);
    });
    return window;
}

const window = initWindow();
const element = React.createElement(App);
const pages = window.getPages();
if (pages && pages[0]) {
    pages[0].update(element);
} else {
    window.newPage(element);
}

/// Hot reload support
//@ts-ignore
module.hot && module.hot.accept();

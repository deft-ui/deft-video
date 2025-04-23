import "./app-title.scss"
import {Container, Image, PageContext, Row} from "deft-react";
import React, {useContext, useEffect, useState} from "react";

export function AppTitle() {
    const pageContext = useContext(PageContext)
    const closeIcon = require("../assets/close.svg");
    const restoreIcon = require("../assets/restore.svg");
    const [isMax, setMax] = useState(false);

    function onPressTitle() {
        pageContext.window.drag();
    }
    function onToggleMax() {
        pageContext.window.maximized = !pageContext.window.maximized;
    }

    function onMin() {
        pageContext.window.minimized = true;
    }
    function onClose() {
        pageContext.window.close();
    }

    useEffect(() => {
        function onResize() {
            setMax(pageContext.window.maximized);
        }
        pageContext.window.addEventListener("resize", onResize);
        return () => {
            pageContext.window.removeEventListener("resize", onResize);
        }
    }, []);

    return <Row className="app-title" onMouseDown={onPressTitle}>
        <Container>DeftVideoPlayer</Container>
        <Container className="app-title-btn-group">
            <Container onClick={onMin} className="btn">
                <Container className="btn-min-icon" />
            </Container>
            <Container onClick={onToggleMax} className="btn">
                { isMax && <Image src={restoreIcon} style={{width: 16, height: 16}} /> }
                { !isMax && <Container className="btn-max-icon" /> }
            </Container>
            <Container onClick={onClose} className="btn">
                <Image src={closeIcon} style={{width: 25, height: 25}} />
            </Container>
        </Container>
    </Row>
}
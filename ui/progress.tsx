import {Container} from "deft-react";
import React from "react";

export interface ProgressProps {
    style ?: StyleProps,
    value: number;
    onChange?: (value: number) => void;
}

export default function Progress(props ?: ProgressProps): React.ReactElement {
    const value = props.value || 0;
    const progressPercent = value.toFixed(2);
    const [width, setWidth] = React.useState(0);

    function onBoundsChange(e: IBoundsChangeEvent) {
        setWidth(e.detail.originBounds.width);
    }

    function onClick(e: IMouseEvent) {
        let x = e.detail.offsetX;
        const newProgress = Math.min(x / width, 1.0) * 100;
        console.log({newProgress});
        props.onChange?.(newProgress);
    }

    const style: StyleProps = {
        width: `${progressPercent}%`,
    }

    return <Container
        style={props.style}
        className="progress"
        onBoundsChange={onBoundsChange}
        onClick={onClick}
        cursor="pointer"
    >
        <Container className="track">
            <Container className="thumb" style={style} />
        </Container>
    </Container>
}
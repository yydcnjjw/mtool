import QtQuick 2.12
import QtQuick.Window 2.12
import QtQuick.Shapes 1.12
import demo 1.0

Window {
    visible: true
    visibility: Window.FullScreen

    MouseArea {
        property bool isPressed: false
        property int originX: 0
        property int originY: 0

        anchors.fill: parent
        cursorShape: Qt.CrossCursor
        acceptedButtons: Qt.LeftButton

        onPressed: {
            if (screenEditor.visible) {
                return
            }

            screenEditor.visible = true
            isPressed = true
            originX = mouseX
            originY = mouseY
        }

        onPositionChanged: {
            if (!isPressed) {
                return
            }

            var offsetW = mouseX - originX
            var offsetH = mouseY - originY

            screenEditor.width = Math.abs(offsetW)
            screenEditor.height = Math.abs(offsetH)

            screenEditor.x = originX + (offsetW > 0 ? 0 : offsetW)
            screenEditor.y = originY + (offsetH > 0 ? 0 : offsetH)
        }

        onReleased: {
            cursorShape = Qt.ArrowCursor
            isPressed = false
        }
    }

    Image {
        anchors.fill: parent
        id: screenImage
        source: "image://screen"
    }

    Rectangle {
        anchors.fill: parent
        opacity: 0.5
    }

    ShaderEffectSource {
        id: screenshot
        anchors.fill: screenEditor
        sourceItem: screenImage
        sourceRect: {
            Qt.rect(screenEditor.x,
                    screenEditor.y,
                    screenEditor.width,
                    screenEditor.height)
        }
    }

    Rectangle {
        id: screenEditor
        visible: false

        color: 'transparent'

        border {
            color: resize_area.color
            width: 2
        }

        MouseArea {
            anchors.fill: parent
            drag.target: parent
            hoverEnabled: true

            onEntered: {
                cursorShape = Qt.DragMoveCursor
            }

            onExited: {
                cursorShape = Qt.ArrowCursor
            }

            onDoubleClicked: {
                screenshot.grabToImage(function(result) {
                    Message.call(result.image)
                })
            }
        }

        QtObject {
            id: resize_area
            readonly property int size: 18
            readonly property color color: 'steelblue'
        }

        function resizeLimitWidth() {
            if(width < resize_area.size)
                width = resize_area.size
        }

        function resizeLimitHeight() {
            if(height < resize_area.size)
                height = resize_area.size
        }


        function resizeLeft(mouseX) {
            width = width - mouseX
            x = x + mouseX
            resizeLimitWidth()
        }

        function resizeRight(mouseX) {
            width = width + mouseX
            resizeLimitWidth()
        }

        function resizeTop(mouseY) {
            height = height - mouseY
            y = y + mouseY
            resizeLimitHeight()
        }

        function resizeBottom(mouseY) {
            height = height + mouseY
            resizeLimitHeight()
        }


        function resizeLeftTop(mouseX, mouseY) {
            resizeLeft(mouseX)
            resizeTop(mouseY)
        }

        function resizeRightTop(mouseX, mouseY) {
            resizeRight(mouseX)
            resizeTop(mouseY)
        }

        function resizeLeftBottom(mouseX, mouseY) {
            resizeLeft(mouseX)
            resizeBottom(mouseY)
        }

        function resizeRightBottom(mouseX, mouseY) {
            resizeRight(mouseX)
            resizeBottom(mouseY)
        }

        component ResizeArea : Rectangle {
            property int hoverCursorShape
            property int dragAxis
            signal positionChanged(real mouseX, real mouseY)

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true

                onEntered: {
                    cursorShape = parent.hoverCursorShape
                }
                onExited: {
                    cursorShape = Qt.ArrowCursor
                }

                drag {
                    target: parent
                    axis: parent.dragAxis
                }

                onPositionChanged: {
                    if (!drag.active) {
                        return
                    }
                    parent.positionChanged(mouseX, mouseY)
                }
            }
        }

        ResizeArea {
            width: resize_area.size
            height: parent.height - resize_area.size
            color: 'transparent'
            anchors.horizontalCenter: parent.left
            anchors.verticalCenter: parent.verticalCenter

            hoverCursorShape: Qt.SizeHorCursor
            dragAxis: Drag.XAxis

            onPositionChanged: parent.resizeLeft(mouseX)
        }

        ResizeArea {
            width: resize_area.size
            height: parent.height - resize_area.size
            color: 'transparent'
            anchors.horizontalCenter: parent.right
            anchors.verticalCenter: parent.verticalCenter

            hoverCursorShape: Qt.SizeHorCursor
            dragAxis: Drag.XAxis

            onPositionChanged: parent.resizeRight(mouseX)
        }

        ResizeArea {
            width: parent.width - resize_area.size
            height: resize_area.size
            color: 'transparent'
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.top

            hoverCursorShape: Qt.SizeVerCursor
            dragAxis: Drag.YAxis

            onPositionChanged: parent.resizeTop(mouseY)
        }

        ResizeArea {
            width: parent.width - resize_area.size
            height: resize_area.size
            color: 'transparent'
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.bottom

            hoverCursorShape: Qt.SizeVerCursor
            dragAxis: Drag.YAxis

            onPositionChanged: parent.resizeBottom(mouseY)
        }

        ResizeArea {
            width: resize_area.size
            height: resize_area.size
            radius: resize_area.size
            color: resize_area.color
            anchors.horizontalCenter: parent.left
            anchors.verticalCenter: parent.top

            hoverCursorShape: Qt.SizeAllCursor
            dragAxis: Drag.XAndYAxis

            onPositionChanged: parent.resizeLeftTop(mouseX, mouseY)
        }

        ResizeArea {
            width: resize_area.size
            height: resize_area.size
            radius: resize_area.size
            color: resize_area.color
            anchors.horizontalCenter: parent.right
            anchors.verticalCenter: parent.top

            hoverCursorShape: Qt.SizeAllCursor
            dragAxis: Drag.XAndYAxis

            onPositionChanged: parent.resizeRightTop(mouseX, mouseY)
        }

        ResizeArea {
            width: resize_area.size
            height: resize_area.size
            radius: resize_area.size
            color: resize_area.color
            anchors.horizontalCenter: parent.left
            anchors.verticalCenter: parent.bottom

            hoverCursorShape: Qt.SizeAllCursor
            dragAxis: Drag.XAndYAxis

            onPositionChanged: parent.resizeLeftBottom(mouseX, mouseY)
        }

        ResizeArea {
            width: resize_area.size
            height: resize_area.size
            radius: resize_area.size
            color: resize_area.color
            anchors.horizontalCenter: parent.right
            anchors.verticalCenter: parent.bottom

            hoverCursorShape: Qt.SizeAllCursor
            dragAxis: Drag.XAndYAxis

            onPositionChanged: parent.resizeRightBottom(mouseX, mouseY)
        }
    }

    Shortcut {
        sequence: StandardKey.Quit
        onActivated: Qt.quit()
    }
}

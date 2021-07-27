import QtQuick 2.12
import QtQuick.Window 2.12
import QtQuick.Shapes 1.12

Window {
    visible: true
    // visibility: Window.FullScreen

    Rectangle {
        id: root
        x: 100
        y: 100
        width: 200
        height: 200

        color: "#354682B4"

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
        }

        QtObject {
            id: resize_area
            readonly property int size: 18
            readonly property color color: 'steelblue'
        }

        component ResizeArea : Rectangle {
            property int hoverCursorShape
            property int dragAxis
            signal positionChanged(real mouseX, real mouseY)

            MouseArea {
                id: mouse_area
                anchors.fill: parent
                hoverEnabled: true

                onEntered: {
                    cursorShape = parent.hoverCursorShape
                }
                onExited: {
                    cursorShape = Qt.ArrowCursor
                }

                drag {
                    target: parent; axis: parent.dragAxis
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
            height: parent.height - width
            color: 'transparent'
            anchors.horizontalCenter: parent.left
            anchors.verticalCenter: parent.verticalCenter

            hoverCursorShape: Qt.SizeHorCursor
            dragAxis: Qt.XAxis

            Rectangle {
                width: 2
                height: parent.height
                color: resize_area.color
                anchors.centerIn: parent
            }

            onPositionChanged: {
                root.width = root.width - mouseX
                root.x = root.x + mouseX
                if(root.width < resize_area.size)
                    root.width = resize_area.size
            }

        }

//        Rectangle {
//            width: resize_area.size
//            height: parent.height - width
//            color: 'transparent'
//            anchors.horizontalCenter: parent.left
//            anchors.verticalCenter: parent.verticalCenter

//            Rectangle {
//                width: 2
//                height: parent.height
//                color: resize_area.color
//                anchors.centerIn: parent
//            }

//            MouseArea {
//                anchors.fill: parent
//                hoverEnabled: true

//                onEntered: {
//                    cursorShape = Qt.SizeHorCursor
//                }
//                onExited: {
//                    cursorShape = Qt.ArrowCursor
//                }

//                drag {
//                    target: parent; axis: Drag.XAxis
//                }
//                onMouseXChanged: {
//                    if (!drag.active) {
//                        return
//                    }

//                    root.width = root.width - mouseX
//                    root.x = root.x + mouseX
//                    if(root.width < resize_area.size)
//                        root.width = resize_area.size
//                }
//            }
//        }

        Rectangle {
            width: resize_area.size
            height: resize_area.size
            color: resize_area.color
            anchors.horizontalCenter: parent.left
            anchors.verticalCenter: parent.top

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true

                onEntered: {
                    cursorShape = Qt.SizeAllCursor
                }
                onExited: {
                    cursorShape = Qt.ArrowCursor
                }

                drag {
                    target: parent; axis: Drag.XAndYAxis
                }

                onPositionChanged: {
                    if (!drag.active) {
                        return
                    }

                    root.width = root.width - mouseX
                    root.x = root.x + mouseX
                    if(root.width < 30)
                        root.width = 30

                    root.height = root.height - mouseY
                    root.y = root.y + mouseY
                    if(root.height < resize_area.size)
                        root.height = resize_area.size
                }
            }
        }
    }

    Shortcut {
        sequence: StandardKey.Quit
        onActivated: Qt.quit()
    }
}

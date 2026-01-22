export interface A2UIProps {
    [key: string]: any;
}

export interface A2UIComponent {
    id: string;
    type: 'text' | 'button' | 'card' | 'list';
    props?: A2UIProps;
    children?: A2UIComponent[];
}

export interface A2UIScreen {
    id: string;
    title?: string;
    components: A2UIComponent[];
}

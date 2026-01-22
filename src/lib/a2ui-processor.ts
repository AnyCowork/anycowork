/**
 * A2UI Message Processor
 * Processes A2UI messages and builds UI screens according to the A2UI protocol
 */

export interface A2UIMessage {
  beginRendering?: {
    surfaceId: string;
    root: string;
    styles?: {
      font?: string;
      primaryColor?: string;
    };
  };
  surfaceUpdate?: {
    surfaceId: string;
    components: Array<{
      id: string;
      weight?: number;
      component: Record<string, any>;
    }>;
  };
  dataModelUpdate?: {
    surfaceId: string;
    path?: string;
    contents: Array<{
      key: string;
      valueString?: string;
      valueNumber?: number;
      valueBoolean?: boolean;
      valueMap?: Array<any>;
      valueList?: Array<any>;
    }>;
  };
  deleteSurface?: {
    surfaceId: string;
  };
}

export interface A2UISurface {
  id: string;
  root?: string;
  styles?: {
    font?: string;
    primaryColor?: string;
  };
  components: Map<string, any>;
  dataModel: any;
}

export class A2UIProcessor {
  private surfaces: Map<string, A2UISurface> = new Map();

  /**
   * Process an array of A2UI messages and return the resulting surfaces
   */
  processMessages(messages: A2UIMessage[]): Map<string, A2UISurface> {
    for (const message of messages) {
      if (message.beginRendering) {
        this.handleBeginRendering(message.beginRendering);
      } else if (message.surfaceUpdate) {
        this.handleSurfaceUpdate(message.surfaceUpdate);
      } else if (message.dataModelUpdate) {
        this.handleDataModelUpdate(message.dataModelUpdate);
      } else if (message.deleteSurface) {
        this.handleDeleteSurface(message.deleteSurface);
      }
    }
    return this.surfaces;
  }

  private handleBeginRendering(msg: NonNullable<A2UIMessage['beginRendering']>) {
    const surface: A2UISurface = {
      id: msg.surfaceId,
      root: msg.root,
      styles: msg.styles,
      components: new Map(),
      dataModel: {},
    };
    this.surfaces.set(msg.surfaceId, surface);
  }

  private handleSurfaceUpdate(msg: NonNullable<A2UIMessage['surfaceUpdate']>) {
    const surface = this.surfaces.get(msg.surfaceId);
    if (!surface) {
      console.warn(`Surface ${msg.surfaceId} not found for update`);
      return;
    }

    // Update components
    for (const comp of msg.components) {
      surface.components.set(comp.id, comp);
    }
  }

  private handleDataModelUpdate(msg: NonNullable<A2UIMessage['dataModelUpdate']>) {
    const surface = this.surfaces.get(msg.surfaceId);
    if (!surface) {
      console.warn(`Surface ${msg.surfaceId} not found for data update`);
      return;
    }

    // Build data model from contents
    const data = this.buildDataModel(msg.contents);
    
    // Update at path or root
    const path = msg.path || '/';
    if (path === '/') {
      surface.dataModel = data;
    } else {
      this.setDataAtPath(surface.dataModel, path, data);
    }
  }

  private handleDeleteSurface(msg: NonNullable<A2UIMessage['deleteSurface']>) {
    this.surfaces.delete(msg.surfaceId);
  }

  private buildDataModel(contents: Array<any>): any {
    const result: any = {};
    
    for (const entry of contents) {
      const key = entry.key;
      
      if (entry.valueString !== undefined) {
        result[key] = entry.valueString;
      } else if (entry.valueNumber !== undefined) {
        result[key] = entry.valueNumber;
      } else if (entry.valueBoolean !== undefined) {
        result[key] = entry.valueBoolean;
      } else if (entry.valueMap !== undefined) {
        result[key] = this.buildDataModel(entry.valueMap);
      } else if (entry.valueList !== undefined) {
        result[key] = entry.valueList.map((item: any) => 
          Array.isArray(item) ? this.buildDataModel(item) : item
        );
      }
    }
    
    return result;
  }

  private setDataAtPath(obj: any, path: string, value: any) {
    const parts = path.split('/').filter(p => p);
    let current = obj;
    
    for (let i = 0; i < parts.length - 1; i++) {
      if (!current[parts[i]]) {
        current[parts[i]] = {};
      }
      current = current[parts[i]];
    }
    
    if (parts.length > 0) {
      current[parts[parts.length - 1]] = value;
    }
  }

  /**
   * Get a specific surface by ID
   */
  getSurface(surfaceId: string): A2UISurface | undefined {
    return this.surfaces.get(surfaceId);
  }

  /**
   * Get all surfaces
   */
  getAllSurfaces(): A2UISurface[] {
    return Array.from(this.surfaces.values());
  }

  /**
   * Clear all surfaces
   */
  clear() {
    this.surfaces.clear();
  }
}

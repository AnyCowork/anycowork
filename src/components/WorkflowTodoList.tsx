/**
 * Workflow TODO List Component
 * 
 * Displays real-time TODO list with:
 * - Visual progress tracking
 * - Status icons
 * - Confirmation dialogs
 * - Human-in-the-loop controls
 */

import React, { useState, useEffect, useCallback } from 'react';
import { CheckCircle, Circle, Clock, AlertCircle, XCircle, SkipForward, Loader2 } from 'lucide-react';

interface WorkflowStep {
  id: string;
  description: string;
  status: 'pending' | 'in_progress' | 'waiting_confirmation' | 'completed' | 'failed' | 'skipped' | 'blocked';
  danger_level: 'safe' | 'warning' | 'critical';
  requires_confirmation: boolean;
  result?: any;
  error?: string;
  started_at?: string;
  completed_at?: string;
  estimated_duration?: number;
  actual_duration?: number;
}

interface TodoList {
  workflow_id: string;
  title: string;
  steps: WorkflowStep[];
  progress: {
    total: number;
    completed: number;
    failed: number;
    in_progress: number;
    pending: number;
    percentage: number;
  };
}

interface ConfirmationRequest {
  step: WorkflowStep;
  message: string;
  danger_level: string;
  timeout: number;
}

interface WorkflowTodoListProps {
  workflowId: string;
  onComplete?: () => void;
}

export const WorkflowTodoList: React.FC<WorkflowTodoListProps> = ({ workflowId, onComplete }) => {
  const [todoList, setTodoList] = useState<TodoList | null>(null);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [confirmationRequest, setConfirmationRequest] = useState<ConfirmationRequest | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  // Connect to WebSocket
  useEffect(() => {
    const websocket = new WebSocket(`ws://localhost:8080/ws/workflows/${workflowId}`);

    websocket.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
    };

    websocket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleWebSocketMessage(message);
    };

    websocket.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
    };

    websocket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    setWs(websocket);

    return () => {
      websocket.close();
    };
  }, [workflowId]);

  const handleWebSocketMessage = useCallback((message: any) => {
    console.log('Received message:', message);

    switch (message.type) {
      case 'connected':
        // Request current status
        ws?.send(JSON.stringify({ type: 'get_status' }));
        break;

      case 'initialized':
      case 'step_started':
      case 'step_completed':
      case 'step_failed':
      case 'step_skipped':
      case 'plan_adapted':
        // Update TODO list
        if (message.todo_list) {
          setTodoList(message.todo_list);
        }
        break;

      case 'confirmation_request':
        // Show confirmation dialog
        setConfirmationRequest({
          step: message.step,
          message: message.message,
          danger_level: message.danger_level,
          timeout: message.timeout
        });
        break;

      case 'workflow_completed':
        // Workflow finished
        if (message.todo_list) {
          setTodoList(message.todo_list);
        }
        onComplete?.();
        break;

      case 'workflow_cancelled':
      case 'workflow_failed':
        // Workflow stopped
        if (message.todo_list) {
          setTodoList(message.todo_list);
        }
        break;

      case 'status_update':
        // Status response
        if (message.data?.todo_list) {
          setTodoList(message.data.todo_list);
        }
        break;
    }
  }, [ws, onComplete]);

  const handleConfirmation = (approved: boolean, skip: boolean = false) => {
    if (!confirmationRequest || !ws) return;

    // Send confirmation response
    ws.send(JSON.stringify({
      type: 'confirmation_response',
      step_id: confirmationRequest.step.id,
      approved,
      skip,
      message: approved ? 'Approved by user' : (skip ? 'Skipped by user' : 'Rejected by user')
    }));

    // Close dialog
    setConfirmationRequest(null);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="w-5 h-5 text-green-500" />;
      case 'in_progress':
        return <Loader2 className="w-5 h-5 text-blue-500 animate-spin" />;
      case 'waiting_confirmation':
        return <Clock className="w-5 h-5 text-yellow-500" />;
      case 'failed':
        return <XCircle className="w-5 h-5 text-red-500" />;
      case 'skipped':
        return <SkipForward className="w-5 h-5 text-gray-400" />;
      case 'blocked':
        return <AlertCircle className="w-5 h-5 text-orange-500" />;
      default:
        return <Circle className="w-5 h-5 text-gray-300" />;
    }
  };

  const getDangerLevelColor = (level: string) => {
    switch (level) {
      case 'critical':
        return 'text-red-600 bg-red-50 border-red-200';
      case 'warning':
        return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      default:
        return 'text-green-600 bg-green-50 border-green-200';
    }
  };

  if (!todoList) {
    return (
      <div className="flex items-center justify-center p-8">
        <Loader2 className="w-8 h-8 animate-spin text-blue-500" />
        <span className="ml-2 text-gray-600">Loading workflow...</span>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-lg p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-800">{todoList.title}</h2>
          <div className={`px-3 py-1 rounded-full text-sm font-medium ${
            isConnected ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
          }`}>
            {isConnected ? '● Connected' : '● Disconnected'}
          </div>
        </div>

        {/* Progress Bar */}
        <div className="mb-4">
          <div className="flex justify-between text-sm text-gray-600 mb-2">
            <span>Progress: {todoList.progress.completed}/{todoList.progress.total} steps</span>
            <span>{todoList.progress.percentage}%</span>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-3">
            <div
              className="bg-blue-500 h-3 rounded-full transition-all duration-500"
              style={{ width: `${todoList.progress.percentage}%` }}
            />
          </div>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-4 gap-4 text-center">
          <div>
            <div className="text-2xl font-bold text-green-600">{todoList.progress.completed}</div>
            <div className="text-xs text-gray-500">Completed</div>
          </div>
          <div>
            <div className="text-2xl font-bold text-blue-600">{todoList.progress.in_progress}</div>
            <div className="text-xs text-gray-500">In Progress</div>
          </div>
          <div>
            <div className="text-2xl font-bold text-gray-600">{todoList.progress.pending}</div>
            <div className="text-xs text-gray-500">Pending</div>
          </div>
          <div>
            <div className="text-2xl font-bold text-red-600">{todoList.progress.failed}</div>
            <div className="text-xs text-gray-500">Failed</div>
          </div>
        </div>
      </div>

      {/* TODO List */}
      <div className="bg-white rounded-lg shadow-lg p-6">
        <h3 className="text-lg font-semibold text-gray-800 mb-4">Tasks</h3>
        <div className="space-y-3">
          {todoList.steps.map((step, index) => (
            <div
              key={step.id}
              className={`flex items-start p-4 rounded-lg border-2 transition-all ${
                step.status === 'in_progress'
                  ? 'border-blue-300 bg-blue-50'
                  : step.status === 'completed'
                  ? 'border-green-200 bg-green-50'
                  : step.status === 'failed'
                  ? 'border-red-200 bg-red-50'
                  : step.status === 'waiting_confirmation'
                  ? 'border-yellow-300 bg-yellow-50'
                  : 'border-gray-200 bg-white'
              }`}
            >
              <div className="flex-shrink-0 mt-1">
                {getStatusIcon(step.status)}
              </div>
              <div className="ml-4 flex-1">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-medium text-gray-500">#{index + 1}</span>
                    <span className="font-medium text-gray-800">{step.description}</span>
                  </div>
                  <span className={`px-2 py-1 rounded text-xs font-medium border ${
                    getDangerLevelColor(step.danger_level)
                  }`}>
                    {step.danger_level}
                  </span>
                </div>
                
                {step.status === 'in_progress' && (
                  <div className="mt-2 text-sm text-blue-600">
                    ⏳ Executing...
                  </div>
                )}
                
                {step.status === 'waiting_confirmation' && (
                  <div className="mt-2 text-sm text-yellow-600">
                    ⏸️ Waiting for confirmation...
                  </div>
                )}
                
                {step.error && (
                  <div className="mt-2 text-sm text-red-600">
                    ❌ Error: {step.error}
                  </div>
                )}
                
                {step.actual_duration && (
                  <div className="mt-2 text-xs text-gray-500">
                    Completed in {step.actual_duration}s
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Confirmation Dialog */}
      {confirmationRequest && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-2xl max-w-md w-full mx-4 p-6">
            <div className="flex items-center mb-4">
              <AlertCircle className={`w-8 h-8 mr-3 ${
                confirmationRequest.danger_level === 'critical'
                  ? 'text-red-500'
                  : confirmationRequest.danger_level === 'warning'
                  ? 'text-yellow-500'
                  : 'text-blue-500'
              }`} />
              <h3 className="text-xl font-bold text-gray-800">Confirmation Required</h3>
            </div>

            <div className={`p-4 rounded-lg mb-4 border-2 ${
              getDangerLevelColor(confirmationRequest.danger_level)
            }`}>
              <p className="font-medium mb-2">{confirmationRequest.message}</p>
              <p className="text-sm">{confirmationRequest.step.description}</p>
            </div>

            <div className="flex space-x-3">
              <button
                onClick={() => handleConfirmation(true)}
                className="flex-1 bg-green-500 hover:bg-green-600 text-white font-medium py-3 px-4 rounded-lg transition-colors"
              >
                ✓ Approve
              </button>
              <button
                onClick={() => handleConfirmation(false, true)}
                className="flex-1 bg-gray-500 hover:bg-gray-600 text-white font-medium py-3 px-4 rounded-lg transition-colors"
              >
                ⏭️ Skip
              </button>
              <button
                onClick={() => handleConfirmation(false)}
                className="flex-1 bg-red-500 hover:bg-red-600 text-white font-medium py-3 px-4 rounded-lg transition-colors"
              >
                ✗ Cancel
              </button>
            </div>

            <div className="mt-4 text-xs text-gray-500 text-center">
              Timeout in {confirmationRequest.timeout}s
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default WorkflowTodoList;

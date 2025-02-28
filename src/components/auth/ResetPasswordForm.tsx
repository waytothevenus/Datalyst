import { useState } from "react";
import { Link } from "react-router";
import Label from "../form/Label";
import Input from "../form/input/InputField";
import Button from "../ui/button/Button";
import { useNavigate } from "react-router";
import { invoke } from "@tauri-apps/api/core";
import { notify } from "../../utils/utils";

export default function ResetPasswordForm() {
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [otp, setOtp] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isOtpVerified, setIsOtpVerified] = useState(false);

  const handleVerifyOtp = async () => {
    if (otp.length > 5) {
      setIsOtpVerified(true);
    }
  };

  const handleResetPassword = async () => {
    if (newPassword !== confirmPassword) {
      notify("Passwords do not match!", "error");
      return;
    }

    setIsSubmitting(true);
    try {
      await invoke("reset_password", { email, otp, new_password: newPassword });
      notify("Password reset successfully! Please sign in now.", "success");
      navigate("/signin");
    } catch (error) {
      notify(new String(error).toString(), "error");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="flex flex-col flex-1">
      <div className="flex flex-col justify-center flex-1 w-full max-w-md mx-auto">
        <div>
          <div className="mb-5 sm:mb-8">
            <h1 className="mb-2 font-semibold text-gray-800 text-title-sm dark:text-white/90 sm:text-title-md">
              Reset Password
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {isOtpVerified
                ? "Enter your new password!"
                : "Enter the OTP sent to your email!"}
            </p>
          </div>
          <div>
            <form>
              <div className="space-y-6">
                {!isOtpVerified && (
                  <>
                    <div>
                      <Label>
                        Email <span className="text-error-500">*</span>{" "}
                      </Label>
                      <Input
                        placeholder="info@gmail.com"
                        value={email}
                        type="email"
                        onChange={(e) => setEmail(e.target.value)}
                      />
                    </div>
                    <div>
                      <Label>
                        OTP <span className="text-error-500">*</span>{" "}
                      </Label>
                      <Input
                        placeholder="Enter OTP"
                        value={otp}
                        onChange={(e) => setOtp(e.target.value)}
                      />
                    </div>
                    <div>
                      <Button
                        disabled={isSubmitting}
                        className="w-full"
                        size="sm"
                        onClick={handleVerifyOtp}
                      >
                        {isSubmitting ? "Verifying..." : "Verify OTP"}
                      </Button>
                    </div>
                  </>
                )}
                {isOtpVerified && (
                  <>
                    <div>
                      <Label>
                        New Password <span className="text-error-500">*</span>{" "}
                      </Label>
                      <Input
                        placeholder="Enter new password"
                        type="password"
                        value={newPassword}
                        onChange={(e) => setNewPassword(e.target.value)}
                      />
                    </div>
                    <div>
                      <Label>
                        Confirm Password{" "}
                        <span className="text-error-500">*</span>{" "}
                      </Label>
                      <Input
                        placeholder="Confirm new password"
                        type="password"
                        value={confirmPassword}
                        onChange={(e) => setConfirmPassword(e.target.value)}
                      />
                    </div>
                    <div>
                      <Button
                        disabled={isSubmitting}
                        className="w-full"
                        size="sm"
                        onClick={handleResetPassword}
                      >
                        {isSubmitting ? "Resetting..." : "Reset Password"}
                      </Button>
                    </div>
                  </>
                )}
              </div>
            </form>

            <div className="mt-5">
              <p className="text-sm font-normal text-center text-gray-700 dark:text-gray-400 sm:text-start">
                Remember your password? {""}
                <Link
                  to="/signin"
                  className="text-brand-500 hover:text-brand-600 dark:text-brand-400"
                >
                  Sign In
                </Link>
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
